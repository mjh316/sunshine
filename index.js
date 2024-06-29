const fs = require("fs");

s =
  '[{"type":"Keyword","value":"prepare","content":"prepare","line":1,"column":8},{"type":"Identifier","value":"rows","content":"rows","line":1,"column":13},{"type":"Keyword","value":"as","content":"as","line":1,"column":16},{"type":"Number","value":"1","content":1.0,"line":1,"column":18},{"type":"Plus","value":"+","content":"+","line":1,"column":20},{"type":"Number","value":"2","content":2.0,"line":1,"column":22},{"type":"Keyword","value":"prepare","content":"prepare","line":2,"column":7},{"type":"Identifier","value":"jobs","content":"jobs","line":2,"column":12},{"type":"Keyword","value":"as","content":"as","line":2,"column":15},{"type":"LeftBracket","value":"[","content":"[","line":2,"column":17},{"type":"Number","value":"2","content":2.0,"line":2,"column":18},{"type":"Comma","value":",","content":",","line":2,"column":19},{"type":"Number","value":"3","content":3.0,"line":2,"column":21},{"type":"Comma","value":",","content":",","line":2,"column":22},{"type":"String","value":"a","content":"a","line":2,"column":26},{"type":"RightBracket","value":"]","content":"]","line":2,"column":27},{"type":"EOF","value":"","content":"","line":2,"column":27}]';

// console.log(JSON.parse(s));

ast = JSON.parse(JSON.parse(fs.readFileSync("./ast.txt", "utf8")));

console.log(ast[ast.length - 2].body[ast[ast.length - 2].body.length - 2].args);
