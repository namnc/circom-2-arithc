# Operations

This document outlines the high-level operations and how we process statements, expressions and gates creation.

## Main Component

- It's a template call.
- Includes initialized variables, likely in an `InitializationBlock`.
- The body is traversed using `traverse_sequence_of_statements`.

## Statements

Each statement in the component's body is handled by `traverse_statement`. These include:

- **DataItem Declaration**: Declaring variables or signals, either as single scalar or array. Here we can just add a `DataItem` based on the type and dimension
- **If-Then-Else**: Evaluates conditions (variables or function calls) with `execute_expression`, then executes the chosen path using `traverse_sequence_of_statements`.
- **Loops (While/For)**: Similar to `if-then-else`, but repeats based on a condition (breaks if it's `false`). Uses `traverse_sequence_of_statements` for the loop body.
- **ConstraintEquality**: Probably only for ZK, not MPC.
- **Return Statement**: In a function body, this statement assigns the result directly to the variable on the left-hand side of the call. For instance, in `a = func()`, the return value of `func()` is assigned to `a`. This also applies when `func()` is part of a larger expression, like in `a = a + func()`, where the return value is used as part of the expression calculation.
- **Assert**: Probably only for ZK, not MPC.
- **Substitution**: Like `a <== b + c`, the right-hand side is an expression processed by `traverse_expression`. This is the primary instance where a substitution statement is executed. If the left-hand side is a variable, we use `execute_expression` for execution instead of `traverse_expression`.
- **Block**: Traversed with `traverse_sequence_of_statements`.
- **LogCall**: For debugging, not used.
- **UnderscoreSubstitution**: Probably an anonymous substitution, to be handled later (if `circomlib-ml` uses it).

## Expressions

Parts of statements, like the right side of a substitution or a flow control (if/loop) condition.

- **Number**: A constant. We return its value and can add it as a named variable in the context for ease in mixed signal-variable expressions, like naming "1" for the value 1.
- **Infix-Op**: If the right-hand side is a variable, we use `execute_infix_op`. If not, we use `traverse_infix_op`. It gets complex when variables are mixed with signals, like if rhs is a signal. In such cases, we use `traverse_infix_op` and might need to create an intermediate signal. For example, in `sum[1] = sum[0] + input_A[0]*input_B[0]`, we generate an intermediate signal for `input_A[0]*input_B[0]`. If one of the operands is a signal, we create an auto-named signal. If both are variables, we simply execute `execute_infix_op` and return the value. This value can be treated as a constant variable in expressions involving signals, like "1" with value 1. The `execute_infix_op` is a straightforward operation, like in `a = b + c` where `b` is `1` and `c` is `2`, resulting in `a` being `3`.
- **Prefix-Op**: Handled like `infix-op`.
- **Inline-Switch**: Acts like a quick If-Then-Else.
- **ParallelOp**: Not addressed.
- **Variable**: Returns an signal id for a signal, or the value for a variable.
- **Call**: We start by identifying if the call is a template or a function.
  - For both templates and functions, we map the defined arguments to their initialized values at the call time.
  - Next, we process the body of the template or function using `traverse_sequence_of_statements`.
  - In the case of a _function_, we're assuming it processes only variables, as previously covered. (We're assuming that a function body doesn't include a template call)
  - For a _template_ call, it's similar to processing the main call. However, there's an additional step of delayed mapping for input and output signals. After traversing the template, we map the template's signals to the caller's signals. For example, in `component c = Template()` where `Template` has input signal `I` and output signal `O`, these are mapped in the caller's code as `c.I = I` and `c.O = O`.
  - This mapping may be going on in the `traverse_sequence_of_statements`, during `execute_delayed_declarations` if it's a complete template.
  - `if is_complete_template { execute_delayed_declarations(program_archive, runtime, actual_node, flags); }`.
- **AnonymousComponent**: Not addressed.
- **ArrayInLine**: Not addressed.
- **Tuple**: Not addressed.
- **UniformArray**: Not addressed.

## Creating Gates and Operations

- Gates are only created when processing `traverse_infix_op`.
- Based on the operation, a specific gate like a fan-in-2 gate may be added to the circuit.
- For example, when we encounter `traverse_infix_op` and it results in `id_1 = id_2 + id_3`, we create an add gate with `id_2` and `id_3` as inputs and `id_1` as the output.

### Special Gates

For the Comparison, Negative/Positive, and Zero Check gates, Circom implements these using an advisory approach. Our strategy should involve identifying these specific gates during the processing of template calls within `traverse expression`. We do this by matching the template name and then substituting them with dedicated gates for comparison, sign checking, and zero equality.
