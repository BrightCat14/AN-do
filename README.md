# an/do

`an-do` is a simple Rust-based build system, like make or ninja. It builds targets based on dependencies and supports rules and patterns.

---

## 1. do.file syntax

### 1.1. Rules (`rule`)

Rules define reusable commands:

```text
rule compile {
    gcc -c $in -o $out
}

rule link {
    gcc $in -o $out
}
```

* `$in` — input files
* `$out` — output files

---

### 1.2. Build targets

```text
app from main.o util.o {
    gcc main.o util.o -o app
}
```

* `app` — the target (output)
* `from main.o util.o` — dependencies (input)
* `{ ... }` — build command

⚠️ **Important:** for empty dependencies just write:

```text
main.c from {
    gcc -c main.c -o main.o
}
```

---

### 1.3. Pattern rules

Pattern rules let you define build templates for multiple files:

```text
*.o from *.c use compile
```

* `*.o` — output files
* `*.c` — input files
* `use compile` — rule to apply

---

### 1.4. Built-in expressions

You can use expressions like `map`:

```text
app from map(src, ".c", ".o") {
    gcc $in -o app
}
```

---

## 2. Parser and lexer

* `Lexer` — converts the text into tokens (`Word`, `{`, `}`, `KwFrom`, `KwUse`, `$Variable`, etc.)
* `Parser` — turns tokens into an AST (list of `Stmt`) for building the dependency graph

---

## 3. Core structures

* `Graph` — dependency graph
* `HashStore` — stores file hashes to check if rebuild is needed
* `BuildRule` — a build rule for one target
* `PatternRule` — a template rule
* `RuleDef` — a reusable command definition

---

## 4. Building

```bash
./an-do.exe       # normal build
./an-do.exe --dry # dry run, shows what would be built
```

* The parser builds a graph from do.file
* `parallel_build` executes commands in dependency order
* `--dry` just prints what would run without actually executing

---

## 5. Example do.file

Correct version without double `{}`:

```text
main.o from {
    gcc -c main.c -o main.o
}

util.o from {
    gcc -c util.c -o util.o
}

app from main.o util.o {
    gcc main.o util.o -o app
}

lib from main.o util.o {
    gcc main.o util.o -o lib
}

rule compile {
    gcc -c $in -o $out
}

rule link {
    gcc $in -o $out
}

*.o from *.c use compile

app from map(src, ".c", ".o") {
    gcc $in -o app
}
```

---

## 6. Tips

1. For empty dependencies, just leave an empty block `{}` for the command.
2. Use `rule` for reusable commands.
3. Pattern rules (`*.o from *.c use compile`) reduce repetitive build rules.
4. `--dry` is great for debugging the build graph without running commands.