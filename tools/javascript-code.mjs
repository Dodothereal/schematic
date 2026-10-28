// Preserve source offsets while hiding literal data and comments. Template
// expressions remain executable; their surrounding HTML/code samples do not.
export function executableCode(source) {
  const out = source.split("");
  const blank = (a, b) => {
    for (let k = a; k < b; k++)
      if (source[k] !== "\n" && source[k] !== "\r") out[k] = " ";
  };
  function quote(i, q) {
    const start = i++;
    while (i < source.length) {
      if (source[i] === "\\") {
        i += 2;
        continue;
      }
      if (source[i++] === q) break;
    }
    blank(start, i);
    return i;
  }
  function template(i) {
    let start = i++;
    while (i < source.length) {
      if (source[i] === "\\") {
        i += 2;
        continue;
      }
      if (source[i] === "`") {
        blank(start, ++i);
        return i;
      }
      if (source[i] === "$" && source[i + 1] === "{") {
        blank(start, i + 2);
        i = scan(i + 2, true);
        start = i;
        continue;
      }
      i++;
    }
    blank(start, i);
    return i;
  }
  function scan(i, expression = false) {
    let depth = 0;
    while (i < source.length) {
      const c = source[i],
        next = source[i + 1];
      if (c === '"' || c === "'") {
        i = quote(i, c);
        continue;
      }
      if (c === "`") {
        i = template(i);
        continue;
      }
      if (c === "/" && next === "/") {
        const start = i;
        i = source.indexOf("\n", i + 2);
        if (i < 0) i = source.length;
        blank(start, i);
        continue;
      }
      if (c === "/" && next === "*") {
        const start = i,
          end = source.indexOf("*/", i + 2);
        i = end < 0 ? source.length : end + 2;
        blank(start, i);
        continue;
      }
      if (c === "/") {
        const preceding = source.slice(0, i).trimEnd(),
          last = preceding.at(-1);
        if (
          !last ||
          /[=(:,!&|?;{}]/.test(last) ||
          /\b(?:return|throw|case|yield|await)\s*$/.test(preceding)
        ) {
          const start = i++;
          let group = false;
          while (i < source.length) {
            const token = source[i++];
            if (token === "\\") {
              i++;
              continue;
            }
            if (token === "[") group = true;
            else if (token === "]") group = false;
            else if (token === "/" && !group) break;
          }
          while (/[a-z]/i.test(source[i] || "") && i < source.length) i++;
          blank(start, i);
          continue;
        }
      }
      if (c === "{") depth++;
      if (c === "}") {
        if (expression && depth === 0) {
          blank(i, i + 1);
          return i + 1;
        }
        depth--;
      }
      i++;
    }
    return i;
  }
  scan(0);
  return out.join("");
}
