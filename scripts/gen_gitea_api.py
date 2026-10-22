#!/usr/bin/env python3
"""Generate the Rust DTO and route layers from Gitea's OpenAPI 3 specification.

Input : gitea/templates/swagger/v1-openapi3.generated.json
Output: crates/shared/src/models.rs      (every schema in components.schemas)
        crates/backend/src/api/routes.rs (router wiring for every operation)
        crates/backend/src/api/ops.rs    (operation metadata table)

Run from the repository root:  python3 scripts/gen_gitea_api.py
"""

from __future__ import annotations

import json
import keyword
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPEC = ROOT / "gitea/templates/swagger/v1-openapi3.generated.json"
OUT_MODELS = ROOT / "crates/shared/src/models.rs"
OUT_WIRE = ROOT / "crates/shared/src/wire.rs"
OUT_ROUTES = ROOT / "crates/backend/src/api/routes.rs"
OUT_OPS = ROOT / "crates/backend/src/api/ops.rs"

RUST_KEYWORDS = {
    "as", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
    "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
    "super", "trait", "true", "type", "unsafe", "use", "where", "while", "async",
    "await", "abstract", "become", "box", "do", "final", "macro", "override",
    "priv", "try", "typeof", "unsized", "virtual", "yield",
}


def snake(name: str) -> str:
    s = re.sub(r"[^0-9a-zA-Z_]", "_", name)
    s = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", s)
    s = re.sub(r"_{2,}", "_", s).strip("_").lower()
    if not s or s[0].isdigit():
        s = "f_" + s
    if keyword.iskeyword(s) or s in RUST_KEYWORDS:
        s += "_"
    return s


def pascal(name: str) -> str:
    parts = re.split(r"[^0-9a-zA-Z]+", name)
    out = "".join(p[:1].upper() + p[1:] for p in parts if p)
    if not out or out[0].isdigit():
        out = "T" + out
    return out


def api_path(path: str) -> str:
    """Absolute API path: the specification's paths are relative to `/api/v1`."""
    return "/api/v1" + path


# Gitea registers these with a trailing wildcard (`m.Get("/*", ...)`,
# `m.Get("/{base}/*", ...)`, `m.Get("/raw/*", ...)`, ...), so the last parameter
# swallows the remaining path segments. Gitea documents the parameter as a single
# token (`{filepath}`), but the handler must still see a slash-joined tail.
#
# Axum's `{*rest}` catch-all cannot coexist with a single-parameter sibling
# (`/contents/{p2}` vs `/contents/{*rest}` is a route conflict), so only the
# paths whose wildcard has no single-segment sibling are registered as
# catch-alls. The rest are matched segment-wise by the dispatcher, which joins
# the tail back together before the handler reads it.
CATCH_ALL_PATHS = {
    "/repos/{owner}/{repo}/archive/{archive}",
    "/repos/{owner}/{repo}/compare/{basehead}",
    "/repos/{owner}/{repo}/contents-ext/{filepath}",
    "/repos/{owner}/{repo}/contents/{filepath}",
    "/repos/{owner}/{repo}/editorconfig/{filepath}",
    "/repos/{owner}/{repo}/git/refs/{ref}",
    "/repos/{owner}/{repo}/media/{filepath}",
    "/repos/{owner}/{repo}/pulls/{base}/{head}",
    "/repos/{owner}/{repo}/raw/{filepath}",
    "/repos/{owner}/{repo}/wiki/page/{pageName}",
    "/repos/{owner}/{repo}/wiki/revisions/{pageName}",
}


def axum_route(path: str) -> str:
    """Rewrite a documented path into one Axum accepts.

    Three rewrites are needed. Axum allows a single parameter per segment, but
    Gitea documents paths with a literal inside a parameter segment
    (`{sha}.{diffType}`), so such segments collapse into one parameter. Axum
    rejects two routes whose shapes differ only in parameter *name*
    (`/git/commits/{sha}` vs `/git/commits/{sha_diff_type}`), so parameters are
    named by position instead. And a documented wildcard parameter becomes a
    real catch-all (`{*pN}`) when no single-segment sibling shares its prefix.

    The documented path stays in the operation table and is what the dispatcher
    captures against, so none of these rewrites changes the wire contract.
    """
    segments = []
    index = 0
    raw = path.strip("/").split("/")
    for position, segment in enumerate(raw):
        names = re.findall(r"\{([^}]*)\}", segment)
        if not names:
            segments.append(segment)
            continue
        is_last = position == len(raw) - 1
        if is_last and path in CATCH_ALL_PATHS:
            segments.append(f"{{*p{index}}}")
        else:
            segments.append(f"{{p{index}}}")
        index += 1
    return api_path("/" + "/".join(segments))


def doc_line(text, indent: str) -> str:
    text = " ".join(str(text).split())
    if not text:
        return ""
    return f"{indent}/// {text.replace('*/', '* /')}\n"


class Emitter:
    """Turns OpenAPI schemas into Rust source."""

    def __init__(self, spec: dict):
        self.spec = spec
        self.schemas: dict = spec["components"]["schemas"]
        self.inline: dict[str, dict] = {}
        self.boxed: set[tuple[str, str]] = set()

    def type_name(self, name: str) -> str:
        t = pascal(name)
        if t in RUST_KEYWORDS:
            t += "Dto"
        return t

    def find_cycles(self) -> None:
        graph: dict[str, set[str]] = {}
        for name, schema in self.schemas.items():
            refs: set[str] = set()
            for prop in (schema.get("properties") or {}).values():
                refs |= self._direct_refs(prop)
            graph[self.type_name(name)] = refs

        WHITE, GRAY, BLACK = 0, 1, 2
        color = {n: WHITE for n in graph}

        def dfs(node: str, stack: list[str]) -> None:
            color[node] = GRAY
            stack.append(node)
            for nxt in graph.get(node, ()):
                if nxt not in color:
                    continue
                if color[nxt] == GRAY:
                    idx = stack.index(nxt) if nxt in stack else 0
                    for a, b in zip(stack[idx:], stack[idx + 1:]):
                        self.boxed.add((a, b))
                    self.boxed.add((stack[-1], nxt))
                elif color[nxt] == WHITE:
                    dfs(nxt, stack)
            stack.pop()
            color[node] = BLACK

        for n in list(graph):
            if color[n] == WHITE:
                dfs(n, [])

    def _direct_refs(self, schema) -> set[str]:
        if not isinstance(schema, dict):
            return set()
        if "$ref" in schema:
            return {self.type_name(schema["$ref"].rsplit("/", 1)[-1])}
        if "allOf" in schema and len(schema["allOf"]) == 1:
            return self._direct_refs(schema["allOf"][0])
        if schema.get("type") == "array":
            return set()
        out: set[str] = set()
        for value in (schema.get("properties") or {}).values():
            out |= self._direct_refs(value)
        return out

    def rust_type(self, schema, parent: str, field: str, required: bool) -> str:
        if not isinstance(schema, dict):
            schema = {}
        if "$ref" in schema:
            base = self.type_name(schema["$ref"].rsplit("/", 1)[-1])
            pair = (self.type_name(parent), base)
            if pair in self.boxed:
                inner = f"Box<{base}>"
                return inner if required else f"Option<{inner}>"
            return base if required else f"Option<{base}>"

        if "allOf" in schema:
            refs = [p for p in schema["allOf"] if isinstance(p, dict) and "$ref" in p]
            if len(refs) == 1:
                inner = self.type_name(refs[0]["$ref"].rsplit("/", 1)[-1])
                if schema.get("nullable") or not required:
                    return f"Option<{inner}>"
                return inner
            return "serde_json::Value" if required else "Option<serde_json::Value>"

        for key in ("oneOf", "anyOf"):
            if key in schema:
                refs = [p for p in schema[key] if isinstance(p, dict) and "$ref" in p]
                if len(refs) == 1 and len(schema[key]) == 1:
                    return self.rust_type(refs[0], parent, field, required)
                return "serde_json::Value" if required else "Option<serde_json::Value>"

        t = schema.get("type")
        if t is None:
            if "properties" in schema or "additionalProperties" in schema:
                t = "object"
            else:
                return "serde_json::Value" if required else "Option<serde_json::Value>"
        if isinstance(t, list):
            t = next((x for x in t if x != "null"), "string")

        if t == "array":
            items = schema.get("items") or {}
            inner = f"Vec<{self.rust_type(items, parent, field, True)}>"
            return inner if required else f"Option<{inner}>"

        if t == "object":
            ap = schema.get("additionalProperties")
            if isinstance(ap, dict) and ap:
                inner = self.rust_type(ap, parent, field, True)
                base = f"std::collections::HashMap<String, {inner}>"
                return base if required else f"Option<{base}>"
            if schema.get("properties"):
                nested = f"{pascal(parent)}{pascal(field)}"
                self.inline[nested] = {"name": nested, "schema": schema, "parent": parent}
                return nested if required else f"Option<{nested}>"
            return "serde_json::Value" if required else "Option<serde_json::Value>"

        if "enum" in schema:
            nested = f"{pascal(parent)}{pascal(field)}"
            self.inline[nested] = {"name": nested, "schema": schema, "parent": parent}
            return nested if required else f"Option<{nested}>"

        base = {
            "string": "String",
            "boolean": "bool",
            "integer": {"int64": "i64", "int32": "i32", "uint64": "u64", "uint32": "u32"}.get(
                schema.get("format", "int64"), "i64"
            ),
            "number": {"float": "f32", "double": "f64"}.get(schema.get("format", "double"), "f64"),
            "null": "serde_json::Value",
        }.get(t, "serde_json::Value")

        if schema.get("nullable") and required:
            return f"Option<{base}>"
        return base if required else f"Option<{base}>"

    def emit_enum(self, name: str, schema: dict, out: list[str]) -> None:
        variants: list[tuple[str, object]] = []
        seen: set[str] = set()
        for value in schema["enum"]:
            if value is None:
                continue
            vname = pascal(str(value)) if not str(value).isdigit() else f"V{value}"
            if not vname or vname in seen:
                continue
            seen.add(vname)
            variants.append((vname, value))
        if not variants:
            out.append(f"pub type {name} = String;\n\n")
            return
        out.append("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]\n")
        out.append(f"pub enum {name} {{\n")
        for i, (vname, value) in enumerate(variants):
            if i == 0:
                out.append("    #[default]\n")
            if vname != str(value):
                out.append(f'    #[serde(rename = "{value}")]\n')
            out.append(f"    {vname},\n")
        out.append("}\n\n")

    def emit_struct(self, name: str, schema: dict, out: list[str]) -> None:
        props = schema.get("properties") or {}
        required = set(schema.get("required") or [])
        out.append(doc_line(schema.get("description", ""), ""))
        out.append("#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]\n")
        out.append(f"pub struct {name} {{\n")
        if not props:
            out.append(
                "    #[serde(flatten)]\n"
                "    pub extra: std::collections::HashMap<String, serde_json::Value>,\n"
            )
        for prop, pschema in sorted(props.items()):
            if not isinstance(pschema, dict):
                pschema = {}
            fname = snake(prop)
            is_required = prop in required
            ty = self.rust_type(pschema, name, prop, is_required)
            out.append(doc_line(pschema.get("description", ""), "    "))
            attrs = []
            if fname != prop:
                attrs.append(f'rename = "{prop}"')
            # Optional fields must tolerate absence on the wire; required ones are
            # always emitted so responses stay shaped like Gitea's.
            if not is_required:
                attrs.append("default")
                if ty.startswith("Option<"):
                    attrs.append('skip_serializing_if = "Option::is_none"')
            if attrs:
                out.append(f"    #[serde({', '.join(attrs)})]\n")
            out.append(f"    pub {fname}: {ty},\n")
        out.append("}\n\n")

    def generate_models(self) -> str:
        self.find_cycles()
        out: list[str] = [
            "//! Generated from Gitea's OpenAPI 3 specification.\n",
            "//!\n",
            "//! Source: `gitea/templates/swagger/v1-openapi3.generated.json`\n",
            "//! Regenerate with `python3 scripts/gen_gitea_api.py`.\n",
            "//!\n",
            "//! These are the wire types Gitea serves on `/api/v1`; the backend builds\n",
            "//! them from its stores and the frontend consumes them unchanged.\n",
            "\n#![allow(clippy::all)]\n\nuse serde::{Deserialize, Serialize};\n\n",
        ]

        pending = dict(self.schemas)
        emitted: set[str] = set()
        while pending:
            batch = pending
            pending = {}
            for name, schema in sorted(batch.items()):
                tname = self.type_name(name)
                if tname in emitted:
                    continue
                emitted.add(tname)
                if "enum" in schema and not schema.get("properties"):
                    self.emit_enum(tname, schema, out)
                else:
                    self.emit_struct(tname, schema, out)
            for iname, info in sorted(self.inline.items()):
                if iname not in emitted:
                    pending[iname] = info["schema"]

        return "".join(out)

    def generate_wire(self) -> str:
        """`from_wire`/`as_wire` conversions for every generated enum."""
        out: list[str] = [
            "//! Wire-value conversions for the generated enums.\n",
            "//!\n",
            "//! Kept apart from `models.rs` so the generated type definitions stay\n",
            "//! purely mechanical. Regenerate with `python3 scripts/gen_gitea_api.py`.\n\n",
            "use crate::models::*;\n\n",
        ]
        # Enums declared inline in a property are emitted into `models.rs` as
        # named types, so they need conversions too.
        candidates: dict[str, dict] = dict(self.schemas)
        for name, info in self.inline.items():
            candidates.setdefault(name, info["schema"])

        for name, schema in sorted(candidates.items()):
            if "enum" not in schema or schema.get("properties"):
                continue
            variants: list[tuple[str, str]] = []
            seen: set[str] = set()
            for value in schema["enum"]:
                if value is None:
                    continue
                vname = pascal(str(value)) if not str(value).isdigit() else f"V{value}"
                if not vname or vname in seen:
                    continue
                seen.add(vname)
                variants.append((vname, str(value)))
            if not variants:
                continue
            tname = self.type_name(name)
            default = variants[0][0]
            out.append(f"impl {tname} {{\n")
            out.append("    /// The value as it appears on the wire.\n")
            out.append("    pub const fn as_wire(&self) -> &'static str {\n        match self {\n")
            for vname, value in variants:
                out.append(f'            Self::{vname} => "{value}",\n')
            out.append("        }\n    }\n\n")
            out.append("    /// Parse a wire value, falling back to the first variant.\n")
            out.append("    pub fn from_wire(value: &str) -> Self {\n        match value {\n")
            for vname, value in variants:
                out.append(f'            "{value}" => Self::{vname},\n')
            out.append(f"            _ => Self::{default},\n")
            out.append("        }\n    }\n}\n\n")
        return "".join(out)

    def generate_routes(self) -> tuple[str, str]:
        ops: list[dict] = []
        for path, methods in self.spec["paths"].items():
            for method, op in methods.items():
                if method not in ("get", "post", "put", "patch", "delete", "head", "options"):
                    continue
                body = None
                for param in op.get("parameters", []):
                    if param.get("in") == "body":
                        body = self._schema_name(param.get("schema", {}))
                if op.get("requestBody"):
                    content = op["requestBody"].get("content", {})
                    for ctype in ("application/json", "text/plain", "*/*"):
                        if ctype in content:
                            body = self._schema_name(content[ctype].get("schema", {}))
                            break
                resp = None
                for code in ("200", "201", "202"):
                    content = (op.get("responses", {}).get(code) or {}).get("content", {})
                    if "application/json" in content:
                        resp = self._schema_name(content["application/json"].get("schema", {}))
                        break
                ops.append(
                    {
                        "method": method.upper(),
                        "path": api_path(path),
                        "route": axum_route(path),
                        "id": op.get("operationId", ""),
                        "tag": (op.get("tags") or ["miscellaneous"])[0],
                        "summary": " ".join((op.get("summary") or "").split()).replace('"', "'"),
                        "body": body,
                        "resp": resp,
                        "deprecated": bool(op.get("deprecated")),
                    }
                )

        routes = [
            "//! Generated Axum wiring: one route per Gitea `/api/v1` operation.\n",
            "//!\n",
            f"//! {len(ops)} operations across {len({o['route'] for o in ops})} paths.\n",
            "//! Regenerate with `python3 scripts/gen_gitea_api.py`.\n",
            "\nuse axum::routing::any;\nuse axum::Router;\n\n",
            "use super::dispatch::dispatch;\nuse crate::state::AppState;\n\n",
            "/// Every `/api/v1` path Gitea exposes, served by the shared dispatcher.\n",
            "pub fn api_router() -> Router<AppState> {\n    Router::new()\n",
        ]
        for route in sorted({o["route"] for o in ops}):
            routes.append(f'        .route("{route}", any(dispatch))\n')
        routes.append("}\n")

        ops_src = [
            "//! Generated operation metadata for the Gitea `/api/v1` surface.\n",
            "//!\n",
            "//! Regenerate with `python3 scripts/gen_gitea_api.py`.\n\n",
            "/// One documented API operation.\n",
            "pub struct Op {\n",
            "    pub method: &'static str,\n",
            "    /// The path as documented, used to capture path parameters.\n",
            "    pub path: &'static str,\n",
            "    pub id: &'static str,\n",
            "    pub tag: &'static str,\n",
            "    pub summary: &'static str,\n",
            "    pub body: Option<&'static str>,\n",
            "    pub resp: Option<&'static str>,\n",
            "    pub deprecated: bool,\n",
            "}\n\n",
            f"/// All {len(ops)} operations, in specification order.\n",
            "pub static OPS: &[Op] = &[\n",
        ]
        for o in ops:
            b = f'Some("{o["body"]}")' if o["body"] else "None"
            r = f'Some("{o["resp"]}")' if o["resp"] else "None"
            ops_src.append(
                f'    Op {{ method: "{o["method"]}", path: "{o["path"]}", '
                f'id: "{o["id"]}", tag: "{o["tag"]}", summary: "{o["summary"]}", '
                f'body: {b}, resp: {r}, '
                f"deprecated: {str(o['deprecated']).lower()} }},\n"
            )
        ops_src.append("];\n\n")
        ops_src.append(
            "/// Look up an operation by HTTP method and documented path.\n"
            "pub fn find(method: &str, path: &str) -> Option<&'static Op> {\n"
            "    OPS.iter().find(|op| op.method == method && op.path == path)\n"
            "}\n\n"
            "/// Every tag used by the specification, with its operation count.\n"
            "#[cfg(test)]\n"
            "pub fn tag_counts() -> Vec<(&'static str, usize)> {\n"
            "    let mut out: Vec<(&'static str, usize)> = Vec::new();\n"
            "    for op in OPS {\n"
            "        match out.iter_mut().find(|(t, _)| *t == op.tag) {\n"
            "            Some(entry) => entry.1 += 1,\n"
            "            None => out.push((op.tag, 1)),\n"
            "        }\n"
            "    }\n"
            "    out.sort_by_key(|(_, count)| std::cmp::Reverse(*count));\n"
            "    out\n"
            "}\n"
        )
        return "".join(routes), "".join(ops_src)

    def _schema_name(self, schema) -> str | None:
        if not isinstance(schema, dict):
            return None
        if "$ref" in schema:
            return schema["$ref"].rsplit("/", 1)[-1]
        if "allOf" in schema and schema["allOf"]:
            return self._schema_name(schema["allOf"][0])
        if "items" in schema:
            return self._schema_name(schema["items"])
        if schema.get("type") in ("string", "boolean", "integer"):
            return schema["type"]
        return None


def main() -> None:
    spec = json.loads(SPEC.read_text())
    emitter = Emitter(spec)
    models = emitter.generate_models()
    wire = emitter.generate_wire()
    routes, ops = emitter.generate_routes()

    OUT_MODELS.parent.mkdir(parents=True, exist_ok=True)
    OUT_ROUTES.parent.mkdir(parents=True, exist_ok=True)
    OUT_MODELS.write_text(models)
    OUT_WIRE.write_text(wire)
    OUT_ROUTES.write_text(routes)
    OUT_OPS.write_text(ops)

    print(f"models.rs : {models.count(chr(10))} lines, {len(emitter.schemas)} top-level schemas")
    print(f"wire.rs   : {wire.count('from_wire')} enums")
    print(f"routes.rs : {routes.count('.route(')} axum routes "
          f"({len(spec['paths'])} documented paths)")
    print(f"ops.rs    : {ops.count('Op {')} operations")


if __name__ == "__main__":
    main()
