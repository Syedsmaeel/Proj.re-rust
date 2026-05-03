# Getting started

> **Warning** These bindings are **experimental**, which means they can change
> at any time or be removed outright; nevertheless the plan is to provide a
> stable external C API to the Hoffman language and the Hoffman store.

The language library allows evaluating Hoffman expressions and interacting with Hoffman
language values. The Hoffman store API is still rudimentary, and only allows
initialising and connecting to a store for the Hoffman language evaluator to
interact with.

Currently there are two ways to interface with the Hoffman language evaluator
programmatically:

1. Embedding the evaluator
2. Writing language plug-ins

Embedding means you link the Hoffman C API libraries in your program and use them from
there. Adding a plug-in means you make a library that gets loaded by the Hoffman
language evaluator, specified through a configuration option.

Many of the components and mechanisms involved are not yet documented, therefore
please refer to the [Hoffman source code](https://github.com/HoffmanOS/hoffman/) for
details. Additions to in-code documentation and the reference manual are highly
appreciated.

The following examples, for simplicity, don't include error handling. See the
[Handling errors](@ref errors) section for more information.

# Embedding the Hoffman Evaluator{#hoffman_evaluator_example}

In this example we programmatically start the Hoffman language evaluator with a
dummy store (that has no store paths and cannot be written to), and evaluate the
Hoffman expression `builtins.hoffmanVersion`.

**main.c:**

```C
#include <hoffman_api_util.h>
#include <hoffman_api_expr.h>
#include <hoffman_api_value.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// NOTE: This example lacks all error handling. Production code must check for
// errors, as some return values will be undefined.

void my_get_string_cb(const char * start, unsigned int n, void * user_data)
{
    *((char **) user_data) = strdup(start);
}

int main()
{
    hoffman_libexpr_init(NULL);

    Store * store = hoffman_store_open(NULL, "dummy://", NULL);
    EvalState * state = hoffman_state_create(NULL, NULL, store); // empty search path (HOFFMAN_PATH)
    Value * value = hoffman_alloc_value(NULL, state);

    hoffman_expr_eval_from_string(NULL, state, "builtins.hoffmanVersion", ".", value);
    hoffman_value_force(NULL, state, value);

    char * version;
    hoffman_get_string(NULL, value, my_get_string_cb, &version);
    printf("Hoffman version: %s\n", version);

    free(version);
    hoffman_gc_decref(NULL, value);
    hoffman_state_free(state);
    hoffman_store_free(store);
    return 0;
}
```

**Usage:**

```ShellSession
$ gcc main.c $(pkg-config hoffman-expr-c --libs --cflags) -o main
$ ./main
Hoffman version: 2.17
```

# Writing a Hoffman language plug-in

In this example we add a custom primitive operation (_primop_) to `builtins`. It
will increment the argument if it is an integer and throw an error otherwise.

**plugin.c:**

```C
#include <hoffman_api_util.h>
#include <hoffman_api_expr.h>
#include <hoffman_api_value.h>

void increment(void* user_data, hoffman_c_context* ctx, EvalState* state, Value** args, Value* v) {
    hoffman_value_force(NULL, state, args[0]);
    if (hoffman_get_type(NULL, args[0]) == HOFFMAN_TYPE_INT) {
      hoffman_init_int(NULL, v, hoffman_get_int(NULL, args[0]) + 1);
    } else {
      hoffman_set_err_msg(ctx, HOFFMAN_ERR_UNKNOWN, "First argument should be an integer.");
    }
}

void hoffman_plugin_entry() {
  const char* args[] = {"n", NULL};
  PrimOp *p = hoffman_alloc_primop(NULL, increment, 1, "increment", args, "Example custom built-in function: increments an integer", NULL);
  hoffman_register_primop(NULL, p);
  hoffman_gc_decref(NULL, p);
}
```

**Usage:**

```ShellSession
$ gcc plugin.c $(pkg-config hoffman-expr-c --libs --cflags) -shared -o plugin.so
$ hoffman --plugin-files ./plugin.so repl
hoffman-repl> builtins.increment 1
2
```
