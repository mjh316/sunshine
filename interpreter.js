const fs = require("fs");
ast = JSON.parse(JSON.parse(fs.readFileSync("./ast.txt", "utf8")));

// console.log(ast);

class ReturnException extends Error {
  constructor(value) {
    super();
    this.value = value;
  }
}

class Interpreter {
  error(msg) {
    throw new Error("Interpreter Error: " + msg);
  }

  inScope(scope, name) {
    return Object.keys(scope).includes(name);
  }

  run(ast, scope) {}

  evaluate(value, scope) {
    switch (value.type) {
      case "Var": {
        if (!this.inScope(scope, value.name)) {
          this.error(`Variable ${value.name} not found in scope`);
        }

        return scope[value.name];
      }
      case "Unary": {
        const operations = {
          "!": (a) => !a,
        };
        return operations[value.operator](this.evaluate(value.value, scope));
      }
      case "Binary": {
        const operations = {
          "<": (a, b) => a < b,
          ">": (a, b) => a > b,
          "<=": (a, b) => a <= b,
          ">=": (a, b) => a >= b,
          "==": (a, b) => a == b,
          "!=": (a, b) => a != b,
          "+": (a, b) => a + b,
          "-": (a, b) => a - b,
          "*": (a, b) => a * b,
          "/": (a, b) => a / b,
          "%": (a, b) => a % b,
          "&&": (a, b) => a && b,
          "||": (a, b) => a || b,
        };

        return operations[value.operator](
          this.evaluate(value.left, scope),
          this.evaluate(value.right, scope)
        );
      }
      case "Literal": {
        return value.value;
      }
      case "Array": {
        return value.value.map((expr) => this.evaluate(expr, scope));
      }
      case "Instance": {
        if (!this.inScope(scope, value.name)) {
          this.error(`Struct ${value.name} not found in scope`);
        }

        const instanceConstructor = scope[value.name];
        const fields = {};
        for (let [field, fieldValue] of Object.entries(value.members)) {
          fields[field] = this.evaluate(fieldValue, scope);
        }
        return instanceConstructor(fields);
      }
      case "Call": {
        const caller = this.evaluate(value.caller, scope);
        if (!caller) {
          this.error(`Function ${value.caller.name} not found in scope`);
        }
        if (typeof caller !== "function") {
          this.error(`Variable ${value.caller.name} is not a function`);
        }
        const args = [];
        for (const arg of value.args) {
          args.push(this.evaluate(arg, scope));
        }
        return caller(args);
      }
      default: {
        this.error(`Unknown expression type: ${value.type}`);
      }
    }
  }

  execute(node, scope) {
    switch (node.type) {
      case "Var":
        scope[node.name] = this.evaluate(node.value, scope);
        return scope;
      case "Set": {
        if (!this.inScope(scope, node.name)) {
          this.error(`Variable ${node.name} not found in scope`);
        }
        scope[node.name] = this.evaluate(node.value, scope);
        return scope;
      }
      case "Struct": {
        scope[node.name] = (fields) => {
          const obj = {};
          for (let field of Object.keys(fields)) {
            if (!node.members.includes(field)) {
              this.error(`Field ${field} not found in struct ${node.name}`);
            }
            obj[field] = fields[field];
          }
          return obj;
        };
        return scope;
      }
      case "Func": {
        let func = (args) => {
          let funcScope = { ...scope };
          for (let i = 0; i < node.args.length; i++) {
            funcScope[node.args[i]] = args[i];
          }

          try {
            this.run(node.body, funcScope);
          } catch (err) {
            if (err instanceof ReturnException) {
              return err.value;
            }
            throw err;
          }
        };
        scope[node.name] = func;
        return scope;
      }
      case "Return":
        throw new ReturnException(this.evaluate(node.value, scope));
      case "While":
      case "For":
      case "Conditional":
      default:
        this.evaluate(node, scope);
    }

    return scope;
  }
}
