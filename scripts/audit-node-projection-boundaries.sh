#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_root=$(CDPATH= cd -- "$script_dir/.." && pwd)

exec python3 - "$repository_root" "$@" <<'PY'
from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path


FORBIDDEN = {"NodeInput", "NodeInputOf", "LayoutInput", "LayoutInputOf", "node_input"}
AGGREGATES = {"NodeInput", "NodeInputOf", "LayoutInput", "LayoutInputOf"}


@dataclass(frozen=True)
class Token:
    text: str
    start: int
    end: int


def mask_range(chars: list[str], start: int, end: int) -> None:
    for index in range(start, end):
        if chars[index] != "\n":
            chars[index] = " "


def raw_string_end(source: str, start: int) -> int | None:
    prefix_end = start
    if source.startswith(("br", "rb"), start):
        prefix_end += 2
    elif source.startswith("r", start):
        prefix_end += 1
    else:
        return None
    hashes = 0
    while prefix_end + hashes < len(source) and source[prefix_end + hashes] == "#":
        hashes += 1
    quote = prefix_end + hashes
    if quote >= len(source) or source[quote] != '"':
        return None
    delimiter = '"' + ("#" * hashes)
    close = source.find(delimiter, quote + 1)
    return len(source) if close < 0 else close + len(delimiter)


def quoted_end(source: str, quote: int, delimiter: str) -> int | None:
    index = quote + 1
    while index < len(source):
        if source[index] == "\n" and delimiter == "'":
            return None
        if source[index] == "\\":
            index += 2
            continue
        if source[index] == delimiter:
            return index + 1
        index += 1
    return len(source) if delimiter == '"' else None


def char_literal_end(source: str, quote: int) -> int | None:
    index = quote + 1
    if index >= len(source) or source[index] == "\n":
        return None
    if source[index] == "\\":
        index += 1
        if index >= len(source):
            return None
        if source[index] == "x":
            index += 3
        elif source[index] == "u" and source[index + 1:index + 2] == "{":
            close = source.find("}", index + 2)
            if close < 0:
                return None
            index = close + 1
        else:
            index += 1
    else:
        index += 1
    return index + 1 if source[index:index + 1] == "'" else None


def lexical_mask(source: str) -> str:
    chars = list(source)
    index = 0
    while index < len(source):
        if source.startswith("//", index):
            end = source.find("\n", index + 2)
            end = len(source) if end < 0 else end
            mask_range(chars, index, end)
            index = end
            continue
        if source.startswith("/*", index):
            depth = 1
            end = index + 2
            while end < len(source) and depth:
                if source.startswith("/*", end):
                    depth += 1
                    end += 2
                elif source.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            mask_range(chars, index, end)
            index = end
            continue
        raw_end = raw_string_end(source, index)
        if raw_end is not None:
            mask_range(chars, index, raw_end)
            index = raw_end
            continue
        if source.startswith('b"', index):
            end = quoted_end(source, index + 1, '"')
            end = len(source) if end is None else end
            mask_range(chars, index, end)
            index = end
            continue
        if source[index] == '"':
            end = quoted_end(source, index, '"')
            end = len(source) if end is None else end
            mask_range(chars, index, end)
            index = end
            continue
        char_quote = index + 1 if source.startswith("b'", index) else index
        if source[char_quote:char_quote + 1] == "'":
            end = char_literal_end(source, char_quote)
            if end is not None:
                mask_range(chars, index, end)
                index = end
                continue
        index += 1
    return "".join(chars)


TOKEN_PATTERN = re.compile(r"[A-Za-z_][A-Za-z0-9_]*|::|->|=>|[^\s]")


def tokenize(source: str) -> list[Token]:
    return [Token(match.group(), match.start(), match.end()) for match in TOKEN_PATTERN.finditer(source)]


def attribute_end(tokens: list[Token], start: int) -> int | None:
    if start + 1 >= len(tokens) or tokens[start].text != "#" or tokens[start + 1].text != "[":
        return None
    depth = 0
    for index in range(start + 1, len(tokens)):
        if tokens[index].text == "[":
            depth += 1
        elif tokens[index].text == "]":
            depth -= 1
            if depth == 0:
                return index + 1
    return None


def gated_item_end(tokens: list[Token], start: int) -> int:
    index = start
    while index < len(tokens):
        end = attribute_end(tokens, index)
        if end is None:
            break
        index = end
    stack: list[str] = []
    matching = {")": "(", "]": "[", "}": "{"}
    for current in range(index, len(tokens)):
        text = tokens[current].text
        if text in "([{":
            stack.append(text)
        elif text in matching:
            if stack and stack[-1] == matching[text]:
                stack.pop()
            if text == "}" and not stack:
                return tokens[current].end
        elif text == ";" and not stack:
            return tokens[current].end
    return tokens[-1].end if tokens else 0


def production_source(source: str) -> str:
    masked = lexical_mask(source)
    chars = list(masked)
    tokens = tokenize(masked)
    exact_cfg_test = ["#", "[", "cfg", "(", "test", ")", "]"]
    for index in range(len(tokens) - len(exact_cfg_test) + 1):
        if [token.text for token in tokens[index:index + len(exact_cfg_test)]] != exact_cfg_test:
            continue
        end = gated_item_end(tokens, index + len(exact_cfg_test))
        mask_range(chars, tokens[index].start, end)
    return "".join(chars)


def line_column(source: str, offset: int) -> tuple[int, int]:
    line = source.count("\n", 0, offset) + 1
    previous = source.rfind("\n", 0, offset)
    return line, offset - previous


def forbidden_tokens(source: str) -> list[Token]:
    return [token for token in tokenize(production_source(source)) if token.text in FORBIDDEN]


def owner_alias_violations(source: str) -> list[tuple[Token, str]]:
    tokens = tokenize(production_source(source))
    violations: list[tuple[Token, str]] = []
    for index, token in enumerate(tokens):
        if token.text == "type":
            end = next((cursor for cursor in range(index + 1, len(tokens)) if tokens[cursor].text == ";"), len(tokens))
            equals = next((cursor for cursor in range(index + 1, end) if tokens[cursor].text == "="), None)
            if equals is not None:
                for candidate in tokens[equals + 1:end]:
                    if candidate.text in AGGREGATES:
                        violations.append((candidate, "complete-input type alias"))
        if token.text != "pub":
            continue
        cursor = index + 1
        if cursor < len(tokens) and tokens[cursor].text == "(":
            depth = 1
            cursor += 1
            while cursor < len(tokens) and depth:
                depth += tokens[cursor].text == "("
                depth -= tokens[cursor].text == ")"
                cursor += 1
        if cursor >= len(tokens) or tokens[cursor].text != "use":
            continue
        end = next((position for position in range(cursor + 1, len(tokens)) if tokens[position].text == ";"), len(tokens))
        for candidate in tokens[cursor + 1:end]:
            if candidate.text in AGGREGATES:
                violations.append((candidate, "complete-input reexport"))
    return violations


def self_test() -> int:
    sample = r'''
#[cfg(test)]
use crate::{NodeInputOf, LayoutInputOf};

#[cfg(test)]
fn hidden() {
    let literal = r###"}; NodeInputOf { /* not a comment */"###;
    let ordinary = "} LayoutInput;";
    let byte = b"node_input }";
    let character = '}';
    /* outer { ; /* nested } ; */ NodeInput */
    let node_input = NodeInputOf::<f32>::default();
}

#[cfg(test)]
mod hidden_module {
    fn nested() { let value: LayoutInput = panic!("}"); }
}

fn production(value: NodeInputOf<f32>) {
    let node_input = value;
}
'''
    observed = [token.text for token in forbidden_tokens(sample)]
    if observed != ["NodeInputOf", "node_input"]:
        print(f"self-test: cfg/literal masking mismatch: {observed}", file=sys.stderr)
        return 1

    lifetimes = "fn visible<'a, 'b>(value: &'a NodeInputOf<f32>, other: &'b u8) {}"
    if [token.text for token in forbidden_tokens(lifetimes)] != ["NodeInputOf"]:
        print("self-test: lifetimes were mistaken for character literals", file=sys.stderr)
        return 1

    ufcs = r'''
fn production<Tree: Compute>() {
    let lookup = <Tree as Compute>::node_input;
    let value = lookup(tree, node);
}
'''
    produced = production_source(ufcs)
    ufcs_tokens = tokenize(produced)
    if [token.text for token in ufcs_tokens if token.text in FORBIDDEN] != ["node_input"]:
        print("self-test: extracted UFCS lookup was masked", file=sys.stderr)
        return 1
    lookup_offsets = [token.start for token in ufcs_tokens if token.text == "lookup"]
    node_offset = next(token.start for token in ufcs_tokens if token.text == "node_input")
    if len(lookup_offsets) != 2 or not (lookup_offsets[0] < node_offset < lookup_offsets[1]):
        print("self-test: extracted UFCS binding or later invocation disappeared", file=sys.stderr)
        return 1

    aliases = "type Complete = NodeInputOf<f32>; pub(crate) use crate::LayoutInput;"
    reasons = [reason for _, reason in owner_alias_violations(aliases)]
    if reasons != ["complete-input type alias", "complete-input reexport"]:
        print(f"self-test: complete-input owner aliases escaped: {reasons}", file=sys.stderr)
        return 1
    print("node-projection boundary audit self-test: PASS")
    return 0


def direct_non_input(directory: Path) -> list[Path]:
    return sorted(path for path in directory.glob("*.rs") if path.name != "input.rs")


def recursive_non_input(directory: Path) -> list[Path]:
    return sorted(path for path in directory.rglob("*.rs") if path.name != "input.rs")


def selected_paths(root: Path, mode: str) -> tuple[list[Path], list[Path]]:
    shared = root / "src/node_projection.rs"
    if mode == "scroll":
        return direct_non_input(root / "src/scroll"), [shared, root / "src/scroll/input.rs"]
    if mode == "block-inline":
        paths = direct_non_input(root / "src/block") + [root / "src/inline/mod.rs"]
        return sorted(set(paths)), [shared, root / "src/block/input.rs", root / "src/inline/input.rs"]
    if mode == "flex":
        return direct_non_input(root / "src/flex"), [shared, root / "src/flex/input.rs"]
    if mode == "grid-container":
        paths = [
            root / "src/grid/topology.rs",
            root / "src/grid/tracks/mod.rs",
            root / "src/grid/tracks/validation.rs",
            root / "src/grid/tracks/ordinary.rs",
            root / "src/grid/tracks/flexible.rs",
        ]
        return paths, [shared, root / "src/grid/input.rs"]
    if mode == "grid":
        return recursive_non_input(root / "src/grid"), [shared, root / "src/grid/input.rs"]
    if mode == "all":
        paths = (
            direct_non_input(root / "src/scroll")
            + direct_non_input(root / "src/block")
            + [root / "src/inline/mod.rs"]
            + direct_non_input(root / "src/flex")
            + recursive_non_input(root / "src/grid")
        )
        owners = [
            shared,
            root / "src/scroll/input.rs",
            root / "src/block/input.rs",
            root / "src/inline/input.rs",
            root / "src/flex/input.rs",
            root / "src/grid/input.rs",
        ]
        return sorted(set(paths)), owners
    raise ValueError(mode)


def audit(root: Path, mode: str) -> int:
    paths, owners = selected_paths(root, mode)
    violations: list[str] = []
    for path in paths:
        relative = path.relative_to(root).as_posix()
        if not path.is_file():
            violations.append(f"{relative}: missing fixed audit path")
            continue
        source = path.read_text(encoding="utf-8")
        for token in forbidden_tokens(source):
            line, column = line_column(source, token.start)
            violations.append(f"{relative}:{line}:{column}: forbidden production token `{token.text}`")
    for path in owners:
        relative = path.relative_to(root).as_posix()
        if not path.is_file():
            violations.append(f"{relative}: missing complete-input owner")
            continue
        source = path.read_text(encoding="utf-8")
        for token, reason in owner_alias_violations(source):
            line, column = line_column(source, token.start)
            violations.append(f"{relative}:{line}:{column}: {reason} `{token.text}`")
    if violations:
        print("\n".join(violations), file=sys.stderr)
        return 1
    print(f"node-projection boundary audit ({mode}): PASS ({len(paths)} production paths)")
    return 0


def main() -> int:
    root = Path(sys.argv[1]).resolve()
    arguments = sys.argv[2:]
    if arguments == ["--self-test"]:
        return self_test()
    modes = {"scroll", "block-inline", "flex", "grid-container", "grid", "all"}
    if len(arguments) != 1 or arguments[0] not in modes:
        print("usage: audit-node-projection-boundaries.sh --self-test|scroll|block-inline|flex|grid-container|grid|all", file=sys.stderr)
        return 2
    return audit(root, arguments[0])


raise SystemExit(main())
PY
