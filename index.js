s =
  '[{"type":"Number","value":"2","content":2.0,"line":1,"column":2},{"type":"Plus","value":"+","content":"+","line":1,"column":4},{"type":"Number","value":"4","content":4.0,"line":1,"column":6},{"type":"LeftBracket","value":"[","content":"[","line":2,"column":1},{"type":"Number","value":"2","content":2.0,"line":2,"column":2},{"type":"Comma","value":",","content":",","line":2,"column":3},{"type":"Number","value":"3","content":3.0,"line":2,"column":5},{"type":"RightBracket","value":"]","content":"]","line":2,"column":6},{"type":"EOF","value":"","content":"","line":2,"column":6}]';

console.log(JSON.parse(s));
