const fs = require("fs");
const stdlib = require("./stdlib");
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

  run(ast, scope) {
    // console.log("scope", scope);
    for (const node of ast) {
      scope = this.execute(node, scope);
    }
    return scope;
  }

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

        let left = this.evaluate(value.left, scope);
        let right = this.evaluate(value.right, scope);
        let result = operations[value.operator](left, right);

        // console.log(
        //   "left",
        //   left,
        //   "right",
        //   right,
        //   "result",
        //   result,
        //   value.operator,
        //   operations[value.operator](left, right)
        // );
        return result;
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
      case "Get": {
        if (!this.inScope(scope, value.caller)) {
          this.error(`Variable ${value.caller} not found in scope`);
        }

        const caller = this.evaluate(value.caller, scope);

        let getter;
        if (value.isExpr) {
          getter = caller[this.evaluate(value.property, scope)];
        } else {
          getter = caller[value.property];
        }

        if (getter instanceof Function) return getter.bind(caller);
        return getter;
      }
      default: {
        this.error(`Unknown expression type: ${value.type}`);
      }
    }
  }

  execute(node, scope) {
    console;
    switch (node.type) {
      case "Var":
        scope[node.name] = this.evaluate(node.value, scope);
        // console.log(
        //   "node.name",
        //   node.name,
        //   "value",
        //   scope[node.name],
        //   "scope",
        //   scope
        // );
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
          for (let [i, param] of node.params.entries()) {
            funcScope[param] = args[i];
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
      case "While": {
        // TODO: this.evaluate or this.execute??
        while (this.execute(node.condition, scope)) {
          this.run(node.body, scope);
          //   console.log("condition: ", this.execute(node.condition, scope));
        }
        break;
      }
      case "For": {
        const localScope = {
          ...scope,
          [node.id]: this.evaluate(node.range[0], scope),
        };

        while (localScope[node.id] < this.evaluate(node.range[1], scope)) {
          this.run(node.body, localScope);
          localScope[node.id]++;
        }
        break;
      }
      case "Conditional": {
        if (this.evaluate(node.condition, scope)) {
          this.run(node.body, scope);
        } else {
          for (const other of node.otherwise) {
            this.execute(other, scope);
          }
        }
        break;
      }
      case "Set": {
        if (!this.inScope(scope, node.caller)) {
          this.error(`Variable ${node.caller} not found in scope`);
        }
        scope[node.caller][node.property] = this.evaluate(node.value, scope);
        return scope;
      }
      default:
        return this.evaluate(node, scope);
    }

    return scope;
  }
}

const interpreter = new Interpreter();
try {
  interpreter.run(ast, stdlib);
} catch (err) {
  console.error(err);
}
