
function getScrollBarWidth() {
  let el = document.createElement("div");
  el.style.cssText = "overflow:scroll; visibility:hidden; position:absolute;";
  document.body.appendChild(el);
  let width = el.offsetWidth - el.clientWidth;
  el.remove();
  return width;
}

class TrackSizingParser {
  static INITIAL_CHAR_REGEX = /[a-z-A-Z0-9]/;
  static TOKEN_CHAR_REGEX = /[-\.a-z-A-Z0-9%]/;

  constructor(input, options = { allowFrUnits: true }) {
    this.input = input;
    this.index = 0;
    this.options = options;
  }

  parseList() {
    return this._parseItemList(' ', null);
  }

  parseSingleItem() {
    return this._parseItem();
  }

  _parseItemList(separator, terminator = null) {
    if (!separator) throw new Error('No terminator passed');
    let tokenList = [];
    // console.debug('Parse List', this.index, this.input.slice(this.index));

    while (this.index < this.input.length) {
      const char = this.input[this.index];
      // console.debug(this.index, char);

      // Skip whitespace
      if (char === ' ') { this.index++; continue; }

      if (terminator && char === terminator) {
        return tokenList;
      }

      if (char === '[') {
        const names = this._parseLineNames();
        const previous = tokenList[tokenList.length - 1];
        if (previous?.kind === 'subgrid') previous.lineNames.push(names);
        else tokenList.push({ kind: 'line-names', names });
        continue;
      }

      if (TrackSizingParser.INITIAL_CHAR_REGEX.test(char)) {
        const token = this._parseItem();
        tokenList.push(token);

        const nextChar = this.input[this.index];
        if ((terminator && nextChar === terminator) || !terminator && !nextChar) {
          return tokenList;
        } else {
          this.index++;
          continue;
        }
      }

      throw new Error(`Invalid start of token ${char}`);
    }

    return tokenList;
  }

  _parseItem() {
    let token = '';
    // console.debug('Parse Item', this.index, this.input.slice(this.index));

    while (this.index < this.input.length) {
      const char = this.input[this.index];
      // console.debug(this.index, char);

      if (TrackSizingParser.TOKEN_CHAR_REGEX.test(char)) {
        token += char;
        this.index++;
        continue;
      }

      if (char === '(') {
        if (['calc', 'min', 'max', 'clamp'].includes(token)) {
          return { kind: 'scalar', ...this._parseSizingItem(token) };
        }
        if (!['fit-content', 'minmax', 'repeat'].includes(token)) {
          throw new Error(`Unsupported grid track sizing function ${token}`);
        }
        this.index++;
        const args = this._parseItemList(',', ')');
        this.index++;
        return { kind: 'function', name: token, arguments: args };
      }

      if (token === 'subgrid') return { kind: 'subgrid', lineNames: [] };
      return { kind: 'scalar', ...this._parseScalarItem(token) };
    }
    if (token === 'subgrid') return { kind: 'subgrid', lineNames: [] };
    return { kind: 'scalar', ...this._parseScalarItem(token) };

  }

  _parseSizingItem(name) {
    const body = this._parseBalancedParenthesized();
    const dimension = parseSizingDimension(`${name}(${body})`, { allowFrUnits: this.options.allowFrUnits });
    if (!dimension) throw new Error(`Invalid scalar grid track sizing function ${name}(${body})`);
    return dimension;
  }

  _parseBalancedParenthesized() {
    if (this.input[this.index] !== '(') throw new Error('Expected parenthesized calc value');
    this.index++;
    let depth = 1;
    let body = '';
    while (this.index < this.input.length) {
      const char = this.input[this.index];
      this.index++;
      if (char === '(') {
        depth++;
        body += char;
        continue;
      }
      if (char === ')') {
        depth--;
        if (depth === 0) return body;
        body += char;
        continue;
      }
      body += char;
    }
    throw new Error('Unterminated calc grid track sizing function');
  }

  _parseLineNames() {
    this.index++;
    let names = '';
    while (this.index < this.input.length && this.input[this.index] !== ']') {
      names += this.input[this.index];
      this.index++;
    }
    if (this.input[this.index] !== ']') throw new Error('Unterminated grid line name list');
    this.index++;
    return names.trim() === '' ? [] : names.trim().split(/\s+/);
  }

  _parseScalarItem(item) {
    const res = parseRepetition(item) || parseSizingDimension(item, { allowFrUnits: this.options.allowFrUnits });
    if (!res) throw new Error(`Invalid scalar grid track sizing function ${item}`);
    return res;
  }

}

function parseViewportConstraint(e, boundingRect) {
  if (e.parentNode.classList.contains('viewport')) {
    const parentStyle = getComputedStyle(e.parentElement);
    const hostInlineSize = parentStyle.writingMode === 'horizontal-tb'
      ? boundingRect.width
      : boundingRect.height;
    return {
      width: parseDimension(e.parentNode.style.width || 'max-content'),
      height: parseDimension(e.parentNode.style.height || 'max-content'),
      rootContext: 'flex-item',
      parentWritingMode: parentStyle.writingMode,
      parentDirection: parentStyle.direction,
      hostInlineSize,
    }
  } else {
    return {
      width: rootFillsBrowserViewport(e, boundingRect) ? px(window.innerWidth) : { unit: 'max-content' },
      height: { unit: 'max-content' },
      rootContext: 'root',
    }
  }
}

function rootFillsBrowserViewport(e, boundingRect) {
  if (e.style.width) return false;
  if (e.style.display === 'inline-grid' || e.style.display === 'inline-flex' || e.style.display === 'inline-block') return false;
  return Math.round(boundingRect.width) === window.innerWidth;
}

function parseRepetition(input) {
  if (input === "auto-fill") return { unit: 'auto-fill' };
  if (input === "auto-fit") return { unit: 'auto-fit' };
  if (/^[0-9]*$/.test(input)) return { 'unit': 'integer', value: parseInt(input, 10) };
  return undefined;
}

function parseDimension(input, options = { allowFrUnits: false }) {
  if (!input) return undefined;
  if (typeof input === 'object') return parseTypedOmDimension(input) || input;
  const calc = parseCalcDimension(input);
  if (calc) return calc;
  if (options.allowFrUnits && input.endsWith('fr')) return { unit: 'fraction', value: parseFloat(input.replace('fr', '')) };
  if (input.endsWith('px')) return { unit: 'px', value: parseFloat(input.replace('px', '')) };
  if (input.endsWith('%')) return { unit: 'percent', value: parseFloat(input.replace('%', '')) / 100 };
  if (input === 'auto') return { unit: 'auto' };
  if (input === 'min-content') return { unit: 'min-content' };
  if (input === 'max-content') return { unit: 'max-content' };
  return undefined;
}

const MAX_OWNED_SIZING_FUNCTION_DEPTH = 64;
const OWNED_SIZING_KEYWORDS = new Set([
  'auto',
  'none',
  'content',
  'min-content',
  'max-content',
  'stretch',
  'fit-content',
  'contain',
]);
const OWNED_CALC_SIZE_BASES = new Set([
  'any',
  '100%',
  'auto',
  'none',
  'content',
  'min-content',
  'max-content',
  'stretch',
  'fit-content',
  'contain',
]);
const OWNED_COMPLETE_CSS_NUMBER = /^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?$/;

function parseSizingDimension(input, options = { allowFrUnits: false }) {
  if (!input) return undefined;
  if (typeof input === 'object') {
    return canonicalSizingPercentage(input) || parseTypedOmSizingDimension(input, options);
  }

  const value = input.trim();
  if (!ownedSizingTokenIsValid(value, options.allowFrUnits)) return undefined;

  if (value.endsWith('px')) return { unit: 'px', value: Number(value.slice(0, -2)) };
  if (value.endsWith('%') && !value.includes('(')) {
    return { unit: 'percent', value: Number(value.slice(0, -1)) / 100 };
  }
  if (options.allowFrUnits && value.endsWith('fr')) {
    return { unit: 'fraction', value: Number(value.slice(0, -2)) };
  }
  if (OWNED_SIZING_KEYWORDS.has(value)) return { unit: value };
  if (value.startsWith('calc(')) return { unit: 'calc', value };
  return { unit: 'sizing', value };
}

function canonicalSizingPercentage(value) {
  const prototype = Object.getPrototypeOf(value);
  const isPlainObject = prototype === Object.prototype || prototype === null;
  if (!isPlainObject || value.unit !== 'percent' || !Number.isFinite(value.value)) {
    return undefined;
  }
  return value;
}

function parseTypedOmSizingDimension(value, options) {
  if (!value) return undefined;
  if (value.unit === 'percent' && Number.isFinite(value.value)) {
    return { unit: 'percent', value: value.value / 100 };
  }
  if (value.unit === 'px' && Number.isFinite(value.value)) {
    return { unit: 'px', value: value.value };
  }
  if (value.unit === 'fr' && options.allowFrUnits && Number.isFinite(value.value) && value.value >= 0) {
    return { unit: 'fraction', value: value.value };
  }
  if ((value.unit === 'calc' || value.unit === 'sizing') && typeof value.value === 'string') {
    return parseSizingDimension(value.value, options);
  }
  return parseSizingDimension(value.toString ? value.toString() : '', options);
}

function ownedSizingTokenIsValid(value, allowFrUnits) {
  if (!value) return false;
  if (ownedLengthPercentageIsValid(value)) return true;
  if (OWNED_SIZING_KEYWORDS.has(value)) return true;
  if (allowFrUnits && ownedTrackFlexIsValid(value)) return true;

  const sizingFunction = parseOwnedSizingFunction(value);
  if (!sizingFunction) return false;
  if (sizingFunction.name === 'fit-content') {
    const arguments = splitOwnedSizingArguments(sizingFunction.body);
    return arguments?.length === 1 && ownedSizingCalculationIsValid(arguments[0]);
  }
  if (sizingFunction.name === 'calc-size') {
    const arguments = splitOwnedSizingArguments(sizingFunction.body);
    if (arguments?.length !== 2 || !OWNED_CALC_SIZE_BASES.has(arguments[0])) return false;
    const calculation = ownedCalcSizeCalculation(arguments[1]);
    return calculation.valid && !(arguments[0] === 'any' && calculation.usesSize);
  }
  return ownedSizingCalculationIsValid(value);
}

function ownedLengthPercentageIsValid(value) {
  if (value.endsWith('px')) return ownedFiniteCssNumber(value.slice(0, -2));
  if (value.endsWith('%')) return ownedFiniteCssNumber(value.slice(0, -1));
  return false;
}

function ownedTrackFlexIsValid(value) {
  if (!value.endsWith('fr') || !ownedFiniteCssNumber(value.slice(0, -2))) return false;
  return Number(value.slice(0, -2)) >= 0;
}

function ownedFiniteCssNumber(value) {
  return ownedFixtureNumber(value) !== undefined;
}

function ownedFixtureNumber(value) {
  if (!OWNED_COMPLETE_CSS_NUMBER.test(value)) return undefined;
  const number = Math.fround(Number(value));
  return Number.isFinite(number) ? number : undefined;
}

function parseOwnedSizingFunction(value) {
  const openIndex = value.indexOf('(');
  if (openIndex <= 0 || !/^[a-z-]+$/.test(value.slice(0, openIndex))) return undefined;

  let depth = 0;
  for (let index = openIndex; index < value.length; index++) {
    const char = value[index];
    if (char === '[' || char === ']') return undefined;
    if (char === '(') {
      depth++;
      if (depth > MAX_OWNED_SIZING_FUNCTION_DEPTH) return undefined;
      continue;
    }
    if (char !== ')') continue;
    depth--;
    if (depth < 0) return undefined;
    if (depth === 0) {
      if (index + 1 !== value.length) return undefined;
      return {
        name: value.slice(0, openIndex),
        body: value.slice(openIndex + 1, index),
      };
    }
  }
  return undefined;
}

function splitOwnedSizingArguments(body) {
  const arguments = [];
  let depth = 0;
  let start = 0;
  for (let index = 0; index < body.length; index++) {
    const char = body[index];
    if (char === '(') depth++;
    if (char === ')') {
      depth--;
      if (depth < 0) return undefined;
    }
    if (char === ',' && depth === 0) {
      arguments.push(body.slice(start, index).trim());
      start = index + 1;
    }
  }
  if (depth !== 0) return undefined;
  arguments.push(body.slice(start).trim());
  return arguments.some(argument => !argument) ? undefined : arguments;
}

function ownedSizingCalculationIsValid(value) {
  if (ownedLengthPercentageIsValid(value)) return true;
  const sizingFunction = parseOwnedSizingFunction(value);
  if (!sizingFunction) return false;

  if (sizingFunction.name === 'calc') {
    return ownedAffineCalculation(sizingFunction.body, false, false).valid;
  }
  const arguments = splitOwnedSizingArguments(sizingFunction.body);
  if (!arguments) return false;
  if (sizingFunction.name === 'min' || sizingFunction.name === 'max') {
    return arguments.every(ownedSizingCalculationIsValid);
  }
  if (sizingFunction.name === 'clamp') {
    return arguments.length === 3
      && arguments[1] !== 'none'
      && (arguments[0] === 'none' || ownedSizingCalculationIsValid(arguments[0]))
      && ownedSizingCalculationIsValid(arguments[1])
      && (arguments[2] === 'none' || ownedSizingCalculationIsValid(arguments[2]));
  }
  return false;
}

function ownedCalcSizeCalculation(value) {
  const affine = ownedAffineCalculation(value, true, true);
  if (affine.valid) return affine;

  const sizingFunction = parseOwnedSizingFunction(value);
  if (!sizingFunction) return { valid: false, usesSize: false };
  if (sizingFunction.name === 'calc') {
    return ownedAffineCalculation(sizingFunction.body, true, false);
  }
  const arguments = splitOwnedSizingArguments(sizingFunction.body);
  if (!arguments) return { valid: false, usesSize: false };
  if (sizingFunction.name === 'min' || sizingFunction.name === 'max') {
    const calculations = arguments.map(ownedCalcSizeCalculation);
    return {
      valid: calculations.every(calculation => calculation.valid),
      usesSize: calculations.some(calculation => calculation.usesSize),
    };
  }
  if (sizingFunction.name === 'clamp' && arguments.length === 3 && arguments[1] !== 'none') {
    const calculations = arguments.map((argument, index) => {
      if (argument === 'none' && index !== 1) return { valid: true, usesSize: false };
      return ownedCalcSizeCalculation(argument);
    });
    return {
      valid: calculations.every(calculation => calculation.valid),
      usesSize: calculations.some(calculation => calculation.usesSize),
    };
  }
  return { valid: false, usesSize: false };
}

function ownedAffineCalculation(value, allowSize, allowUnitless) {
  const trimmed = value.trim();
  if (!trimmed) return { valid: false, usesSize: false };

  const tokens = trimmed.split(/\s+/);
  let absolutePx = 0;
  let percentFraction = 0;
  let sizeFraction = 0;
  let usesSize = false;
  let start = 0;
  let sign = 1;

  while (start < tokens.length) {
    const operatorOffset = tokens.slice(start).findIndex(token => token === '+' || token === '-');
    const end = operatorOffset === -1 ? tokens.length : start + operatorOffset;
    if (end === start) return { valid: false, usesSize: false };

    const term = ownedAffineTerm(tokens.slice(start, end), allowSize, allowUnitless);
    if (!term.valid) return { valid: false, usesSize: false };

    absolutePx = Math.fround(absolutePx + Math.fround(term.absolutePx * sign));
    percentFraction = Math.fround(percentFraction + Math.fround(term.percentFraction * sign));
    sizeFraction = Math.fround(sizeFraction + Math.fround(term.sizeFraction * sign));
    if (![absolutePx, percentFraction, sizeFraction].every(Number.isFinite)) {
      return { valid: false, usesSize: false };
    }
    usesSize ||= term.usesSize;

    if (end === tokens.length) break;
    sign = tokens[end] === '+' ? 1 : -1;
    start = end + 1;
    if (start === tokens.length) return { valid: false, usesSize: false };
  }

  return { valid: true, usesSize };
}

function ownedAffineTerm(tokens, allowSize, allowUnitless) {
  if (tokens.length === 1) {
    return ownedAffineAtom(tokens[0], allowSize, allowUnitless);
  }
  if (tokens.length !== 3 || tokens[1] !== '*' || !allowSize) {
    return { valid: false, usesSize: false };
  }

  const factor = tokens[0] === 'size'
    ? ownedFixtureNumber(tokens[2])
    : tokens[2] === 'size'
      ? ownedFixtureNumber(tokens[0])
      : undefined;
  if (factor === undefined) return { valid: false, usesSize: false };
  return {
    valid: true,
    usesSize: true,
    absolutePx: 0,
    percentFraction: 0,
    sizeFraction: factor,
  };
}

function ownedAffineAtom(atom, allowSize, allowUnitless) {
  if (atom.endsWith('px')) {
    const absolutePx = ownedFixtureNumber(atom.slice(0, -2));
    if (absolutePx !== undefined) {
      return { valid: true, usesSize: false, absolutePx, percentFraction: 0, sizeFraction: 0 };
    }
  }
  if (atom.endsWith('%')) {
    const percent = ownedFixtureNumber(atom.slice(0, -1));
    if (percent !== undefined) {
      const percentFraction = Math.fround(percent / 100);
      if (Number.isFinite(percentFraction)) {
        return { valid: true, usesSize: false, absolutePx: 0, percentFraction, sizeFraction: 0 };
      }
    }
  }
  if (allowSize) {
    if (atom === 'size') {
      return { valid: true, usesSize: true, absolutePx: 0, percentFraction: 0, sizeFraction: 1 };
    }
    const factorText = atom.endsWith('*size')
      ? atom.slice(0, -5)
      : atom.startsWith('size*')
        ? atom.slice(5)
        : undefined;
    if (factorText !== undefined) {
      const sizeFraction = ownedFixtureNumber(factorText);
      if (sizeFraction !== undefined) {
        return { valid: true, usesSize: true, absolutePx: 0, percentFraction: 0, sizeFraction };
      }
    }
  }
  if (allowUnitless) {
    const absolutePx = ownedFixtureNumber(atom);
    if (absolutePx !== undefined) {
      return { valid: true, usesSize: false, absolutePx, percentFraction: 0, sizeFraction: 0 };
    }
  }
  return { valid: false, usesSize: false };
}

function parseTypedOmDimension(value) {
  if (!value) return undefined;
  if (value.unit === "percent") return { unit: "percent", value: value.value / 100 };
  if (value.unit === "px") return { unit: "px", value: value.value };
  if (value.unit === "fr") return { unit: "fraction", value: value.value };
  return parseCalcDimension(value.toString ? value.toString() : "");
}

function parseCalcDimension(input) {
  const value = normalizeCalcString(input);
  return value ? { unit: "calc", value } : undefined;
}

function normalizeCalcString(input) {
  if (typeof input !== "string") return "";
  const value = input.trim();
  if (!value.startsWith("calc(") || !value.endsWith(")")) return "";
  return value;
}

function containsCalcFunction(input) {
  return typeof input === "string" && input.includes("calc(");
}

function parseResolvedDimension(input, computedInput) {
  return parseSizingDimension(input) || (input ? parseSizingDimension(computedInput) : undefined);
}

function parseNumber(input) {
  if (input === '' || isNaN(input)) return undefined;
  return Number(input);
}

function parseCssPx(value) {
  if (!value.endsWith("px")) {
    throw new Error(`expected computed px value, got ${value}`);
  }
  return Number(value.slice(0, -2));
}

function resolveLineHeightPx(lineHeight, fontSize) {
  if (lineHeight === "normal") {
    return fontSize * 1.2;
  }
  return parseCssPx(lineHeight);
}

function estimateInlineBaselinePx(fontSize, lineHeight) {
  const fontBaseline = fontSize * 0.8;
  const leading = Math.max(0, lineHeight - fontSize);
  return leading / 2 + fontBaseline;
}

function parseRatio(input) {
  if (!input) return undefined;

  if (input.includes('/')) {
    let [width, height] = input.split("/").map(part => parseFloat(part.trim()));
    if (!width || width < 0 || !height || height <= 0) return undefined;
    return width / height;
  }

  let ratio = parseFloat(input);
  if (!ratio || ratio < 0) return undefined;
  return ratio;
}

function parseEnum(input) {
  if (input) return input;
  return undefined;
}

function parseEdges(edges) {
  const left = parseDimension(edges.left);
  const right = parseDimension(edges.right);
  const top = parseDimension(edges.top);
  const bottom = parseDimension(edges.bottom);

  if (!left && !right && !top && !bottom) return undefined;
  return { left, right, top, bottom };
}

function parseEffectiveMargin(e, computedStyle) {
  const autoEdges = inlineAutoMarginEdges(e, computedStyle);
  const authoredEdges = authoredMarginEdges(e, computedStyle);
  if (!hasAuthoredMarginDeclaration(e, computedStyle) && !Object.values(autoEdges).some(Boolean)) return undefined;

  return parseEdges({
    left: effectiveMarginValue(authoredEdges.left, computedStyle.marginLeft, autoEdges.left),
    right: effectiveMarginValue(authoredEdges.right, computedStyle.marginRight, autoEdges.right),
    top: effectiveMarginValue(authoredEdges.top, computedStyle.marginTop, autoEdges.top),
    bottom: effectiveMarginValue(authoredEdges.bottom, computedStyle.marginBottom, autoEdges.bottom),
  });
}

function effectiveMarginValue(authoredValue, computedValue, isAuto) {
  if (isAuto) return "auto";
  if (authoredValue.trim() !== "auto" && marginValueIsNonInitial(authoredValue)) return authoredValue;
  return marginValueIsNonInitial(computedValue) ? computedValue : "";
}

function parseSize(size) {
  const width = parseSizeDimension(size.width);
  const height = parseSizeDimension(size.height);

  if (!width && !height) return undefined;
  return { width, height };
}

function parseSizeDimension(input) {
  if (!input) return undefined;
  if (typeof input === 'object') return input;
  return parseSizingDimension(input);
}

function px(value) {
  return { unit: 'px', value };
}

function parseElementSize(styleValue, computedStyle) {
  const width = styleValue("width");
  const height = styleValue("height");
  const inlineSize = styleValue("inlineSize");
  const blockSize = styleValue("blockSize");

  if (isVerticalWritingMode(computedStyle.writingMode)) {
    return parseSize({ width: width || blockSize, height: height || inlineSize });
  }
  return parseSize({ width: width || inlineSize, height: height || blockSize });
}

function isVerticalWritingMode(writingMode) {
  return writingMode && (writingMode.startsWith("vertical-") || writingMode.startsWith("sideways-"));
}

function parseGaps(styleValue) {
  const gap = styleValue("gap");
  const rowGap = styleValue("rowGap");
  const columnGap = styleValue("columnGap");
  if (gap) {
    if (typeof gap === 'object') {
      const parsedGap = parseDimension(gap);
      return { row: parsedGap, column: parsedGap };
    }
    const gaps = splitCssComponentValues(gap).map(part => parseDimension(part));
    return { row: gaps[0], column: gaps[1] ?? gaps[0] };
  }
  if (rowGap || columnGap) {
    return { row: parseDimension(rowGap), column: parseDimension(columnGap) };
  }
  return undefined;
}

function splitCssComponentValues(input) {
  const values = [];
  let current = "";
  let depth = 0;
  for (const char of input.trim()) {
    if (/\s/.test(char) && depth === 0) {
      if (current) {
        values.push(current);
        current = "";
      }
      continue;
    }
    if (char === "(") depth++;
    if (char === ")") depth = Math.max(0, depth - 1);
    current += char;
  }
  if (current) values.push(current);
  return values;
}

function cssPropertyName(property) {
  return property.replace(/[A-Z]/g, match => `-${match.toLowerCase()}`);
}

function typedOmStyleValue(e, property) {
  if (!e.computedStyleMap) return undefined;
  const styleMap = e.computedStyleMap();
  if (!styleMap || !styleMap.get) return undefined;
  return parseTypedOmDimension(styleMap.get(cssPropertyName(property)));
}

function inlineAuthoredCalcValue(e, property) {
  return containsCalcFunction(e.style[property]) ? e.style[property] : "";
}


function parseGridTrackDefinitions(input) {
  if (input === '') return undefined;
  return new TrackSizingParser(input).parseList();
}

function parseGridAutoFlow(input) {
  if (!/column/.test(input) && !/row/.test(input) && !/dense/.test(input)) return undefined;
  const direction = /column/.test(input) ? 'column' : 'row';
  const algorithm = /dense/.test(input) ? 'dense' : 'sparse';
  return { direction, algorithm };
}

function parseGridPosition(input) {
  if (input === '') return undefined;
  if (input === 'auto') return { kind: 'auto' };
  if (/^span +\d+$/.test(input)) return { kind: 'span', value: parseInt(input.replace(/[^\d]/g, ''), 10) };
  if (/^-?\d+$/.test(input)) return { kind: 'line', value: parseInt(input, 10) };
  const parts = input.trim().split(/ +/);
  if (parts[0] === 'span') {
    const number = parts.find(part => /^-?\d+$/.test(part));
    const name = parts.find(part => !/^-?\d+$/.test(part) && part !== 'span');
    if (name) return { kind: 'named-span', name, value: number ? parseInt(number, 10) : 0 };
  }
  const name = parts.find(part => !/^-?\d+$/.test(part));
  const number = parts.find(part => /^-?\d+$/.test(part));
  if (name) return { kind: 'named-line', name, value: number ? parseInt(number, 10) : 0 };
  throw new Error(`Unsupported grid placement ${input}`);
}

function brInlineMetricsForElement(e, computedStyle) {
  if (e.tagName === 'BR') {
    const fontSize = parseCssPx(computedStyle.fontSize);
    const lineHeight = resolveLineHeightPx(computedStyle.lineHeight, fontSize);
    const baseline = Math.min(lineHeight, estimateInlineBaselinePx(fontSize, lineHeight));
    return {
      baseline: `${baseline}px`,
      lineHeight: `${lineHeight}px`,
    };
  }
  return undefined;
}

function describeElement(e, expectedElement = null) {

  // Get precise, unrounded dimensions for the current element and it's parent
  let boundingRect = e.getBoundingClientRect();
  let parentBoundingRect = e.parentNode.getBoundingClientRect();

  const computedStyle = getComputedStyle(e);
  const useAuthoredCssRules = expectedElement !== null;
  const styleValue = (property) => useAuthoredCssRules ? authoredStyleValue(e, property, computedStyle) : e.style[property];
  const lengthStyleValue = (property) => {
    const authored = styleValue(property);
    const inlineCalc = inlineAuthoredCalcValue(e, property);
    if (!inlineCalc) return containsCalcFunction(authored) ? "" : authored;
    const typed = typedOmStyleValue(e, property);
    if (typed?.unit === "calc") return typed;
    return inlineCalc;
  };
  const children = describeChildNodes(e, expectedElement);
  const brInlineMetrics = brInlineMetricsForElement(e, computedStyle);

  return {
    tagName: e.tagName.toLowerCase(),
    unsupportedReason: unsupportedElementReason(e, computedStyle) || unsupportedChildNodesReason(e),
    style: {
      display: parseEnum(computedStyle.display),
      boxSizing: parseEnum(computedStyle.boxSizing),

      position: parseEnum(styleValue("position")),
      direction: parseEnum(computedStyle.direction),

      writingMode: parseEnum(computedStyle.writingMode),
      order: computedStyle.order,

      cssFloat: parseEnum(styleValue("cssFloat")),
      clear: parseEnum(styleValue("clear")),

      textAlign: parseEnum(styleValue("textAlign")),
      verticalAlign: parseEnum(styleValue("verticalAlign")),
      fontFamily: parseEnum(computedStyle.fontFamily),
      fontSize: parseDimension(computedStyle.fontSize),
      lineHeight: parseDimension(computedStyle.lineHeight),
      inlineBaseline: brInlineMetrics?.baseline ?? "",
      inlineLineHeight: brInlineMetrics?.lineHeight ?? "",

      flexDirection: parseEnum(styleValue("flexDirection")),
      flexWrap: parseEnum(styleValue("flexWrap")),
      overflowX: parseEnum(computedStyle.overflowX),
      overflowY: parseEnum(computedStyle.overflowY),
      overflowClipMargin: computedStyle.overflowClipMargin,
      scrollbarGutter: computedStyle.scrollbarGutter,
      scrollPaddingTop: computedStyle.scrollPaddingTop,
      scrollPaddingRight: computedStyle.scrollPaddingRight,
      scrollPaddingBottom: computedStyle.scrollPaddingBottom,
      scrollPaddingLeft: computedStyle.scrollPaddingLeft,
      scrollMarginTop: computedStyle.scrollMarginTop,
      scrollMarginRight: computedStyle.scrollMarginRight,
      scrollMarginBottom: computedStyle.scrollMarginBottom,
      scrollMarginLeft: computedStyle.scrollMarginLeft,
      scrollSnapType: computedStyle.scrollSnapType,
      scrollSnapAlign: computedStyle.scrollSnapAlign,
      scrollSnapStop: computedStyle.scrollSnapStop,
      scrollbarWidth: getScrollBarWidth(),

      alignItems: parseEnum(styleValue("alignItems")),
      alignSelf: parseEnum(styleValue("alignSelf")),
      justifyItems: parseEnum(styleValue("justifyItems")),
      justifySelf: parseEnum(styleValue("justifySelf")),

      alignContent: parseEnum(styleValue("alignContent")),
      justifyContent: parseEnum(styleValue("justifyContent")),

      flexGrow: parseNumber(styleValue("flexGrow")),
      flexShrink: parseNumber(styleValue("flexShrink")),
      flexBasis: parseSizingDimension(lengthStyleValue("flexBasis")),

      gridTemplateRows: parseGridTrackDefinitions(lengthStyleValue("gridTemplateRows")),
      gridTemplateColumns: parseGridTrackDefinitions(lengthStyleValue("gridTemplateColumns")),
      gridAutoRows: parseGridTrackDefinitions(lengthStyleValue("gridAutoRows")),
      gridAutoColumns: parseGridTrackDefinitions(lengthStyleValue("gridAutoColumns")),
      gridAutoFlow: parseGridAutoFlow(styleValue("gridAutoFlow")),

      gridRowStart: parseGridPosition(styleValue("gridRowStart")),
      gridRowEnd: parseGridPosition(styleValue("gridRowEnd")),
      gridColumnStart: parseGridPosition(styleValue("gridColumnStart")),
      gridColumnEnd: parseGridPosition(styleValue("gridColumnEnd")),

      gap: parseGaps(lengthStyleValue),

      size: parseElementSize(lengthStyleValue, computedStyle),
      minSize: parseSize({
        width: parseResolvedDimension(lengthStyleValue("minWidth"), computedStyle.minWidth),
        height: parseResolvedDimension(lengthStyleValue("minHeight"), computedStyle.minHeight),
      }),
      maxSize: parseSize({
        width: parseResolvedDimension(lengthStyleValue("maxWidth"), computedStyle.maxWidth),
        height: parseResolvedDimension(lengthStyleValue("maxHeight"), computedStyle.maxHeight),
      }),
      aspectRatio: parseRatio(styleValue("aspectRatio")),

      margin: parseEffectiveMargin(e, computedStyle),

      padding: parseEdges({
        left: lengthStyleValue("paddingLeft"),
        right: lengthStyleValue("paddingRight"),
        top: lengthStyleValue("paddingTop"),
        bottom: lengthStyleValue("paddingBottom"),
      }),

      border: parseEdges({
        left: lengthStyleValue("borderLeftWidth"),
        right: lengthStyleValue("borderRightWidth"),
        top: lengthStyleValue("borderTopWidth"),
        bottom: lengthStyleValue("borderBottomWidth"),
      }),

      inset: parseEdges({
        left: lengthStyleValue("left"),
        right: lengthStyleValue("right"),
        top: lengthStyleValue("top"),
        bottom: lengthStyleValue("bottom"),
      }),
    },

    // The textContent is used for generating intrinsic sizing measure funcs
    // So we're only interested in the text content of leaf nodes
    textContent: e.childElementCount === 0 && e.textContent.length && e.textContent !== "\n" ? e.textContent : undefined,

    // The layout of the node in full precision (floating-point)
    unroundedLayout: {
      width: boundingRect.width,
      height: boundingRect.height,
      x: boundingRect.x - parentBoundingRect.x,
      y: boundingRect.y - parentBoundingRect.y,
      scrollWidth: e.scrollWidth,
      scrollHeight: e.scrollHeight,
      clientWidth: e.clientWidth,
      clientHeight: e.clientHeight,
    },

    // The naively rounded layout of the node. This is equivalent to calling Math.round() on
    // each value in the unrounded layout individually
    naivelyRoundedLayout: {
      width: e.offsetWidth,
      height: e.offsetHeight,
      x: e.offsetLeft + e.parentNode.clientLeft,
      y: e.offsetTop + e.parentNode.clientTop,
      scrollWidth: e.scrollWidth,
      scrollHeight: e.scrollHeight,
      clientWidth: e.clientWidth,
      clientHeight: e.clientHeight,
    },

    // The naive rounding can result in 1px gaps in the layout. Chrome also uses
    // a smarter algorithm, but it doesn't expose the output of that rounding.
    // So we emulate the cumulative edge computation here.
    smartRoundedLayout: {
      width: Math.round(boundingRect.right) - Math.round(boundingRect.left),
      height: Math.round(boundingRect.bottom) - Math.round(boundingRect.top),
      x: Math.round(boundingRect.x - parentBoundingRect.x),
      y: Math.round(boundingRect.y - parentBoundingRect.y),
      scrollWidth: e.scrollWidth,
      scrollHeight: e.scrollHeight,
      clientWidth: e.clientWidth,
      clientHeight: e.clientHeight,
    },

    // Whether the test should enable rounding
    useRounding: e.getAttribute("data-test-rounding") !== "false",

    viewport: parseViewportConstraint(e, boundingRect),

    children,
  };
}

function authoredStyleValue(e, property, computedStyle) {
  if (e.style[property]) return e.style[property];

  let value = "";
  let hadOpaqueSheet = false;
  for (const sheet of Array.from(document.styleSheets)) {
    let rules;
    try {
      rules = sheet.cssRules;
    } catch (_) {
      hadOpaqueSheet = true;
      continue;
    }
    for (const rule of Array.from(rules)) {
      if (rule.type !== CSSRule.STYLE_RULE) continue;
      if (!rule.style[property]) continue;
      try {
        if (e.matches(rule.selectorText)) value = rule.style[property];
      } catch (_) {
        continue;
      }
    }
  }
  if (!value && hadOpaqueSheet) return nonInitialComputedStyleValue(property, computedStyle);
  return value;
}

function nonInitialComputedStyleValue(property, computedStyle) {
  const initial = {
    alignContent: "normal",
    alignItems: "normal",
    alignSelf: "auto",
    clear: "none",
    cssFloat: "none",
    flexBasis: "auto",
    flexDirection: "row",
    flexGrow: "0",
    flexShrink: "1",
    flexWrap: "nowrap",
    gridAutoColumns: "auto",
    gridAutoFlow: "row",
    gridAutoRows: "auto",
    gridColumnEnd: "auto",
    gridColumnStart: "auto",
    gridRowEnd: "auto",
    gridRowStart: "auto",
    gridTemplateColumns: "none",
    gridTemplateRows: "none",
    justifyContent: "normal",
    justifyItems: "normal",
    justifySelf: "auto",
    overflowX: "visible",
    overflowY: "visible",
    position: "static",
    textAlign: "start",
    verticalAlign: "baseline",
    writingMode: "horizontal-tb",
  }[property];
  if (initial === undefined) return "";
  const value = computedStyle[property];
  return value && value !== initial ? value : "";
}

function hasAuthoredMarginDeclaration(e, computedStyle) {
  if (styleDeclarationHasMargin(e.style)) return true;

  let hadOpaqueSheet = false;
  for (const sheet of Array.from(document.styleSheets)) {
    let rules;
    try {
      rules = sheet.cssRules;
    } catch (_) {
      hadOpaqueSheet = true;
      continue;
    }
    for (const rule of Array.from(rules)) {
      if (rule.type !== CSSRule.STYLE_RULE) continue;
      if (!styleDeclarationHasMargin(rule.style)) continue;
      try {
        if (e.matches(rule.selectorText)) return true;
      } catch (_) {
        continue;
      }
    }
  }

  return hadOpaqueSheet && computedMarginIsNonInitial(computedStyle);
}

function authoredMarginEdges(e, computedStyle) {
  const typedOmEdges = typedOmMarginEdges(e, inlineAuthoredMarginHasCalc(e.style));
  if (typedOmEdges) return typedOmEdges;

  const edges = { left: "", right: "", top: "", bottom: "" };
  applyInlineAuthoredMarginDeclarations(edges, e.style, computedStyle);
  return edges;
}

function typedOmMarginEdges(e, allowCalc = false) {
  if (!e.computedStyleMap) return undefined;
  const styleMap = e.computedStyleMap();
  if (!styleMap || !styleMap.get) return undefined;
  return {
    left: typedOmMarginValue(styleMap.get("margin-left"), allowCalc),
    right: typedOmMarginValue(styleMap.get("margin-right"), allowCalc),
    top: typedOmMarginValue(styleMap.get("margin-top"), allowCalc),
    bottom: typedOmMarginValue(styleMap.get("margin-bottom"), allowCalc),
  };
}

function typedOmMarginValue(value, allowCalc = false) {
  const dimension = parseTypedOmDimension(value);
  if (!dimension) return "";
  if (dimension.unit === "percent") return `${dimension.value * 100}%`;
  if (dimension.unit === "px") return `${dimension.value}px`;
  if (dimension.unit === "calc" && allowCalc) return dimension.value;
  return "";
}

function inlineAuthoredMarginHasCalc(style) {
  return styleDeclarationProperties(style).some((property) => {
    return property.startsWith("margin") && containsCalcFunction(styleDeclarationValue(style, property));
  });
}

function applyInlineAuthoredMarginDeclarations(edges, style, computedStyle) {
  for (const property of styleDeclarationProperties(style)) {
    applyInlineAuthoredMarginDeclaration(edges, property, styleDeclarationValue(style, property), computedStyle);
  }
}

function applyInlineAuthoredMarginDeclaration(edges, property, value, computedStyle) {
  if (!marginValueIsNonInitial(value)) return;

  switch (property) {
    case "margin-top":
      edges.top = value;
      return;
    case "margin-right":
      edges.right = value;
      return;
    case "margin-bottom":
      edges.bottom = value;
      return;
    case "margin-left":
      edges.left = value;
      return;
    case "margin-inline-start":
      edges[inlineStartEdge(computedStyle)] = value;
      return;
    case "margin-inline-end":
      edges[inlineEndEdge(computedStyle)] = value;
      return;
    case "margin-inline": {
      const [start, end = start] = splitCssComponentValues(value);
      edges[inlineStartEdge(computedStyle)] = start;
      edges[inlineEndEdge(computedStyle)] = end;
      return;
    }
    case "margin": {
      const parts = splitCssComponentValues(value);
      const [top, right = top, bottom = top, left = right] = parts;
      edges.top = top;
      edges.right = right;
      edges.bottom = bottom;
      edges.left = left;
      return;
    }
  }
}

function styleDeclarationHasMargin(style) {
  return styleDeclarationProperties(style).some((property) => {
    return property.startsWith("margin") && marginValueIsNonInitial(styleDeclarationValue(style, property));
  });
}

function marginValueIsNonInitial(value) {
  if (!value) return false;
  const parts = splitCssComponentValues(value);
  return parts.some((part) => part !== "0" && part !== "0px");
}

function computedMarginIsNonInitial(computedStyle) {
  return ["marginLeft", "marginRight", "marginTop", "marginBottom"].some((property) => {
    return marginValueIsNonInitial(computedStyle[property]);
  });
}

function inlineAutoMarginEdges(e, computedStyle) {
  const edges = { left: false, right: false, top: false, bottom: false };
  applyAutoMarginDeclarations(edges, e.style, computedStyle);
  return edges;
}

function applyAutoMarginDeclarations(edges, style, computedStyle) {
  for (const property of styleDeclarationProperties(style)) {
    applyAutoMarginDeclaration(edges, property, styleDeclarationValue(style, property), computedStyle);
  }
}

function styleDeclarationProperties(style) {
  if (!style || !style.length) return [];
  return Array.from({ length: style.length }, (_, index) => style[index]).filter(Boolean);
}

function styleDeclarationValue(style, property) {
  if (!style) return "";
  return style.getPropertyValue ? style.getPropertyValue(property) : "";
}

function applyAutoMarginDeclaration(edges, property, value, computedStyle) {
  const isAuto = value.trim() === "auto";
  switch (property) {
    case "margin-top":
      edges.top = isAuto;
      return;
    case "margin-right":
      edges.right = isAuto;
      return;
    case "margin-bottom":
      edges.bottom = isAuto;
      return;
    case "margin-left":
      edges.left = isAuto;
      return;
    case "margin-inline-start":
      edges[inlineStartEdge(computedStyle)] = isAuto;
      return;
    case "margin-inline-end":
      edges[inlineEndEdge(computedStyle)] = isAuto;
      return;
    case "margin-inline": {
      const [start, end = start] = splitCssComponentValues(value);
      edges[inlineStartEdge(computedStyle)] = start === "auto";
      edges[inlineEndEdge(computedStyle)] = end === "auto";
      return;
    }
    case "margin": {
      const parts = splitCssComponentValues(value);
      const [top, right = top, bottom = top, left = right] = parts;
      edges.top = top === "auto";
      edges.right = right === "auto";
      edges.bottom = bottom === "auto";
      edges.left = left === "auto";
      return;
    }
  }
}

function inlineStartEdge(computedStyle) {
  const rtl = computedStyle.direction === "rtl";
  switch (computedStyle.writingMode) {
    case "vertical-rl":
    case "vertical-lr":
    case "sideways-rl":
      return rtl ? "bottom" : "top";
    case "sideways-lr":
      return rtl ? "top" : "bottom";
    default:
      return rtl ? "right" : "left";
  }
}

function inlineEndEdge(computedStyle) {
  const start = inlineStartEdge(computedStyle);
  return { left: "right", right: "left", top: "bottom", bottom: "top" }[start];
}

function describeChildNodes(e, expectedElement = null) {
  let children = [];
  let childNodes = Array.from(e.childNodes);
  const layoutReadyInlineRun = hasLayoutReadyInlineFixture(e) && childNodes.some((child, index) => {
    return child.nodeType === Node.ELEMENT_NODE && child.tagName === 'BR' ||
      child.nodeType === Node.TEXT_NODE && shouldSerializeLayoutReadyText(child, childNodes, index, e);
  });
  let visualIndex = 0;
  for (let i = 0; i < childNodes.length; i++) {
    let child = childNodes[i];
    if (child.nodeType === Node.ELEMENT_NODE) {
      const described = describeElement(child, expectedElement);
      if (layoutReadyInlineRun && child.tagName !== 'BR' && isInlineLevel(child)) {
        described.atomicInlineParticipation = {
          bidiLevel: getComputedStyle(child).direction === 'rtl' ? 1 : 0,
          followingBreak: 'prohibited',
        };
      }
      children.push(described);
      if (layoutReadyInlineRun && (child.tagName === 'BR' || isInlineLevel(child))) visualIndex++;
    } else if (
      layoutReadyInlineRun &&
      child.nodeType === Node.TEXT_NODE &&
      shouldSerializeLayoutReadyText(child, childNodes, i, e)
    ) {
      const described = layoutReadyTextNodeData(child, e, i, visualIndex);
      if (described) {
        children.push(described);
        visualIndex++;
      }
    }
  }
  return children;
}

function shouldSerializeLayoutReadyText(node, siblings, index, parent) {
  if (!/^\s*$/.test(node.textContent)) return true;
  return isSignificantInlineWhitespace(node, siblings, index, parent);
}

function layoutReadyTextNodeData(node, parent, segmentId, visualIndex) {
  const range = document.createRange();
  range.selectNodeContents(node);
  const rect = range.getBoundingClientRect();
  const fragmentRects = Array.from(range.getClientRects());
  range.detach();
  if (fragmentRects.length === 0 && rect.width === 0 && rect.height === 0) {
    return undefined;
  }
  if (fragmentRects.length !== 1) {
    throw new Error(`layout-ready text segment ${segmentId} must have exactly one fragment`);
  }

  const parentRect = parent.getBoundingClientRect();
  const root = document.getElementById?.('test-root');
  const rootRect = root?.getBoundingClientRect?.() ?? parentRect;
  const computedStyle = getComputedStyle(parent);
  const fontSize = parseCssPx(computedStyle.fontSize);
  const lineHeight = resolveLineHeightPx(computedStyle.lineHeight, fontSize);
  const baseline = Math.min(lineHeight, estimateInlineBaselinePx(fontSize, lineHeight));
  const vertical = isVerticalWritingMode(computedStyle.writingMode);
  const inlineExtent = vertical ? rect.height : rect.width;
  const whitespace = /^\s+$/.test(node.textContent);

  const finite = [
    rect.x, rect.y, rect.width, rect.height,
    parentRect.x, parentRect.y, rootRect.x, rootRect.y,
    inlineExtent, baseline, lineHeight,
  ];
  if (!finite.every(Number.isFinite) || inlineExtent < 0 || baseline < 0 || lineHeight < baseline) {
    throw new Error(`layout-ready text segment ${segmentId} requires a complete finite tuple`);
  }

  const fragments = fragmentRects.map((fragment) => {
    const x = fragment.x - rootRect.x;
    const y = fragment.y - rootRect.y;
    const width = fragment.width;
    const height = fragment.height;
    const baselineX = vertical ? x + baseline : x;
    const baselineY = vertical ? y : y + baseline;
    if (![x, y, width, height, baselineX, baselineY].every(Number.isFinite) || width < 0 || height < 0) {
      throw new Error(`layout-ready text fragment ${segmentId} requires a complete finite tuple`);
    }
    return {
      sourceSegmentId: segmentId,
      lineIndex: 0,
      visualIndex,
      x,
      y,
      width,
      height,
      baselineX,
      baselineY,
    };
  });

  return {
    layoutInput: 'inline-text',
    inlineSegments: [{
      id: segmentId,
      inlineExtent,
      inlineBaseline: baseline,
      inlineLineHeight: lineHeight,
      bidiLevel: computedStyle.direction === 'rtl' ? 1 : 0,
      whitespaceEdge: whitespace ? 'discard-at-both' : 'preserve',
      followingBreak: whitespace ? 'allowed' : 'prohibited',
    }],
    unroundedLayout: layoutReadyTextLayout(rect, parentRect, false),
    smartRoundedLayout: layoutReadyTextLayout(rect, parentRect, true),
    fragments,
    children: [],
  };
}

function layoutReadyTextLayout(rect, parentRect, rounded) {
  if (rounded) {
    return {
      width: Math.round(rect.right) - Math.round(rect.left),
      height: Math.round(rect.bottom) - Math.round(rect.top),
      x: Math.round(rect.x - parentRect.x),
      y: Math.round(rect.y - parentRect.y),
    };
  }
  return {
    width: rect.width,
    height: rect.height,
    x: rect.x - parentRect.x,
    y: rect.y - parentRect.y,
  };
}

function unsupportedElementReason(e, computedStyle) {
  if (
    e.tagName === 'BR' &&
    isVerticalWritingMode(computedStyle.writingMode) &&
    !hasLayoutReadyVerticalBrFixture(e) &&
    !hasLayoutReadyInlineFixture(e)
  ) {
    return "Unsupported vertical <br> line-break semantics";
  }
  if (
    e.tagName === 'BR' &&
    !hasSupportedBrLineBreakParent(e) &&
    !hasLayoutReadyInlineFixture(e)
  ) {
    return "Unsupported <br> outside block inline-run semantics";
  }
  return undefined;
}

function hasSupportedBrLineBreakParent(e) {
  const parent = e.parentElement;
  if (!parent) return false;
  return getComputedStyle(parent).display === "block";
}

function hasLayoutReadyVerticalBrFixture(e) {
  return e.parentElement?.getAttribute?.('data-surgeist-layout-ready-vertical-br') === 'true';
}

function hasLayoutReadyInlineFixture(e) {
  for (let current = e; current; current = current.parentElement) {
    if (current.getAttribute?.('data-surgeist-layout-ready-inline') === 'true') return true;
  }
  return false;
}

function unsupportedChildNodesReason(e) {
  let childNodes = Array.from(e.childNodes);
  let hasElementChild = childNodes.some(child => child.nodeType === Node.ELEMENT_NODE);
  if (!hasElementChild) return undefined;

  for (let i = 0; i < childNodes.length; i++) {
    let child = childNodes[i];
    if (child.nodeType !== Node.TEXT_NODE) continue;
    if (!/^\s*$/.test(child.textContent) && !hasLayoutReadyInlineFixture(e)) {
      return "Unsupported mixed text/element content";
    }
    if (
      isSignificantInlineWhitespace(child, childNodes, i, e) &&
      !hasLayoutReadyInlineFixture(e)
    ) {
      return "Unsupported mixed text/element content";
    }
  }

  return undefined;
}

function isSignificantInlineWhitespace(node, siblings, index, parent = node.parentElement) {
  if (!/^\s+$/.test(node.textContent)) return false;
  const parentDisplay = parent ? getComputedStyle(parent).display : '';
  if (['grid', 'inline-grid', 'grid-lanes', 'inline-grid-lanes'].includes(parentDisplay)) {
    return false;
  }

  let previous = nearestElementSibling(siblings, index, -1);
  let next = nearestElementSibling(siblings, index, 1);
  return previous && next && isInlineLevel(previous) && isInlineLevel(next);
}

function nearestElementSibling(siblings, index, step) {
  for (let i = index + step; i >= 0 && i < siblings.length; i += step) {
    if (siblings[i].nodeType === Node.ELEMENT_NODE) return siblings[i];
    if (siblings[i].nodeType === Node.TEXT_NODE && !/^\s*$/.test(siblings[i].textContent)) return undefined;
  }
  return undefined;
}

function isInlineLevel(e) {
  let authored = e.style.display;
  let computed = getComputedStyle(e).display;
  return authored.startsWith("inline") || computed.startsWith("inline");
}

function textNodeRect(node) {
  let range = document.createRange();
  range.selectNodeContents(node);
  let rect = range.getBoundingClientRect();
  range.detach();
  return rect;
}

function unsupportedTestData(reason) {
  return { unsupportedReason: reason };
}

function getTestData() {
  const root = document.getElementById('test-root');
  if (!root) {
    const reason = "Unsupported missing #test-root fixture root";
    return JSON.stringify({
      borderBoxLtrData: unsupportedTestData(reason),
      contentBoxLtrData: unsupportedTestData(reason),
      borderBoxRtlData: unsupportedTestData(reason),
      contentBoxRtlData: unsupportedTestData(reason),
    });
  }

  document.body.className = "border-box ltr";
  const borderBoxLtrData = describeElement(root);
  document.body.className = "content-box ltr";
  const contentBoxLtrData = describeElement(root);
  document.body.className = "border-box rtl";
  const borderBoxRtlData = describeElement(root);
  document.body.className = "content-box rtl";
  const contentBoxRtlData = describeElement(root);

  return JSON.stringify({ borderBoxLtrData, contentBoxLtrData, borderBoxRtlData, contentBoxRtlData });
}

// Useful when developing this script. Logs the parsed style to the console when any test fixture is opened in a browser.
window.onload = function () {
  try {
    console.log(describeElement(document.getElementById('test-root')));
  } catch (e) {
    console.error(e);
  }
};
