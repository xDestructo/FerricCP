# FerricCP Systems Architecture & Compiler Theory

FerricCP is designed as a custom compiler frontend. Standard compilers (like GCC or Clang) prioritize generating optimized machine code, often ignoring competitive programming (CP) specific logical errors (like integer overflows or silent uninitialized memory, and so on). FerricCP sits right before the compilation step, acting as a lookout for these semantic and syntactic errors.

To achieve sub-millisecond execution times without sacrificing the ability to understand complex C++ scoping rules, FerricCP abandons traditional linear linting in favor of a **Hybrid Query-Driven Semantic Analyzer** built on highly decoupled Object-Oriented principles.

---

## 1. The Language-Agnostic Core & Tree-sitter

One of the primary architectural goals of FerricCP was ensuring that the core Rust engine remains completely detached from any specific programming language.

### How it Works

Traditional linters parse strings using Regular Expressions. FerricCP utilizes **Tree-sitter**, a parser generator tool that builds *concrete syntax trees* (CST).

The workflow is:

1. The Tree-sitter C backend parses raw source code (C++, Python, etc.).
2. It constructs a Concrete Syntax Tree (CST) and exposes it through a uniform `Node` interface.
3. The Rust engine binds directly to these C structures.

As a result, the FerricCP execution loop does **not** understand what C++, Python, or any other language is. It operates entirely on Tree-sitter's language-independent `Node` interface rather than language-specific parsing logic.


Because the core analyzer works solely on this abstract syntax layer, adding support for an entirely new language (such as Java or Rust) requires **zero modifications** to the core `analyzer.rs` execution loop.

---

## 2. Rule Modularity & The Hybrid Query Engine

To prevent the codebase from becoming a tangled collection of `if`/`else` statements, all rules are strictly decoupled from the execution engine.

In FerricCP, **rules are treated as data.**

Every rule is defined in an external YAML configuration file.

```yaml
id: binary_op_type_mismatch
language: cpp
rule_type: semantic
query: >
  (binary_expression
    left: (identifier)
    right: (identifier)) @violation
```

### The Problem with Traditional AST Traversal

Traditional compiler implementations often rely on the **Visitor Pattern** for static analysis.

A visitor recursively traverses the Abstract Syntax Tree (AST), performing rule evaluation as each node is encountered.  
While effective, this approach tightly couples traversal logic with rule execution, making large rule sets increasingly difficult to maintain and extend.


This approach has several drawbacks:

- **O(N)** traversal cost for every rule.
- Slow startup and analysis times as rule counts eventually increase.
- Tight coupling between traversal logic and rule implementations.
- Any syntax-level modification requires recompiling the analyzer itself.

---

### The Hybrid S-Expression Engine

FerricCP eliminates the traditional Visitor Pattern entirely.

Instead of traversing the AST in Rust, the AST is treated as a searchable database using **Tree-sitter's `QueryCursor`**.

### How it Works

#### 1. Boot-Time Compilation

When FerricCP starts:

- YAML rule files are loaded.
- Every S-expression query is compiled through Tree-sitter's C API.
- The compiled queries become highly optimized native `Query` objects stored in memory.

This compilation occurs only once during startup.

---

#### 2. C-Backed Execution

During source analysis, FerricCP never walks the AST manually.

Instead, it hands each compiled `Query` directly to Tree-sitter's optimized C backend.

---

#### 3. Targeted Node Selection

The C backend searches only for nodes matching the exact structural pattern described by the query.

Instead of manually traversing every node in Rust, FerricCP uses Tree-sitter's optimized C query engine for efficient structural matching. The engine returns an iterator containing only the matching nodes, allowing the Rust layer to operate exclusively on relevant syntax.

Conceptually, this transforms the execution model from:

- **O(N)** tree traversal, where Rust manually walks the entire syntax tree.

into

- **O(1)** event handling per match, where Tree-sitter performs structural matching in optimized native code and Rust simply processes the yielded nodes.

As a result, the analysis layer never performs a full tree traversal itself, reducing unnecessary workload while keeping rule implementations simple, modular, and highly extensible.

---

### Benefits

Because rules exist entirely as modular YAML files:

- New rules can be added without modifying the Rust engine.
- Contributors do not need Rust knowledge to create syntactic rules.
- Rule libraries can scale to thousands of entries without increasing binary complexity.
- Rule packs can be enabled or disabled independently.

---
## 3. Object-Oriented Design

FerricCP's scalability largely comes from how it implements **semantic rules**, rules which require intense state tracking and memory analysis.

The project uses **trait-based polymorphism** (Rust's equivalent of java's interfaces) to enforce the **Open/Closed Principle**.

### The Dispatcher Pattern

When a YAML query identifies a suspicious AST node, the engine itself has no knowledge of how that node should be analyzed.

Instead, it forwards the node to the `RuleDispatcher`.

```rust
pub trait SemanticRule {
    fn check(
        &self,
        node: Node,
        source: &[u8],
        table: &SymbolTable,
        rule: &RuleConfig,
    ) -> Option<Diagnostic>;
}
```

The dispatcher maintains a polymorphic registry:

```rust
HashMap<String, Box<dyn SemanticRule>>
```

It reads the rule ID from YAML, retrieves the corresponding Rust implementation, and invokes it through dynamic dispatch.

### Adding a New Rule

Adding a new semantic rule requires only three steps:

1. Write the YAML rule.
2. Implement the `SemanticRule` trait.
3. Register the implementation inside `rules.rs`.

The core analyzer loop, AST traversal logic, parser integration, and file I/O remain completely untouched.

This architecture provides immense extensibility and scalability.

---

## 4. Compiler Theory 

Semantic analysis cannot rely solely on an Abstract Syntax Tree (AST).

Consider the following example:

```cpp
vector<int> arr(n);
cin >> n;
```

A purely syntactic query can identify the variable `n`, but it has no notion of *program state*. It cannot determine that `n` has not yet been initialized when the vector is allocated.

To solve this problem, FerricCP implements a complete **Symbol Table**, a fundamental compiler data structure responsible for tracking variable bindings, initialization state, lifetimes, lexical scopes, and compiler metadata across the entire source file.

---

### The Language Agnostic Foundation

One of FerricCP's architectural goals is language independence.

Rather than storing C++ specific objects, the base `SymbolTable` is implemented as a language agnostic abstraction. Every identifier is represented internally by a `VariableState` structure containing metadata such as:

- `is_initialized` - whether the variable has been assigned a value.
- `line_declared` - the declaration location used for chronological lookups.
- `declared_type` - the resolved type signature.
- optional language specific metadata (macros, aliases, etc.).

This abstraction allows semantic rules to reason about program state without depending on any language specific parser logic.

---

### Rust Implementation & Config Bootstrapping

Internally, the Symbol Table is backed by Rust's highly optimized:

```rust
HashMap<String, VariableState>
```

This provides constant-time access for nearly all semantic lookups.

However, FerricCP's architecture deliberately avoids hardcoded rule parameters.

During startup, `config.rs` loads every YAML rule and deserializes it into a strongly typed `RuleConfig` structure. Rather than embedding configuration strings throughout the analyzer, each semantic rule receives its corresponding `RuleConfig` through the `RuleDispatcher`.

This provides several advantages:

- Compile time type safety.
- Elimination of magic strings inside the Rust backend.
- Runtime configurable semantic rules.
- Complete separation between rule definitions and execution logic.

The Symbol Table is therefore queried dynamically using validated configuration data rather than manually written constants.

---

### C++ Specific Implementation and Challenges

C++ presents several unique problems for static analysis that require additional compiler-aware handling.

### 1. Lexical Scoping & Variable Shadowing

C++ permits **variable shadowing**, where multiple variables with identical names may exist simultaneously in different scopes.

For example:

```cpp
int ans;

void solve() {
    int ans;
}
```

A naïve symbol table keyed only by variable name would overwrite the global entry, producing incorrect diagnostics.

FerricCP instead fingerprints every declaration using a composite key:

```text
{variable_name}_{line_declared}
```

### Lookup Process

```text
                    Variable: ans (Line 45)
                         │
                         ▼
              HashMap<String, VariableState>
                         │
        ┌────────────────┴────────────────┐
        ▼                                 ▼
    ans_3                            ans_18
    (global)                        (local)
        │                                 │
        └──────────────┬──────────────────┘
                       ▼
        Keep declarations before Line 45
                       │
                       ▼
      Choose largest line_declared value
                       │
                       ▼
        ans_18 → Current VariableState
                       │
                       ▼
          Return to Semantic Rule
```

Whenever a semantic rule evaluates an identifier, it searches for the declaration whose line number is the closest preceding declaration with the same name.

This effectively provides constant-time historical lookups while fully respecting C++ lexical scoping and nested block boundaries.

---

### 2. The Macro Preprocessor Logic

Competitive programming codebases frequently rely on aggressive preprocessor macros such as:

```cpp
#define int long long
#define rep(i,a,b) for(int i=a; i<b; i++)
#define vi vector<int>
```

Tree-sitter parses the raw source code **before** the GCC preprocessor expands these directives.

Consequently, many traditional analyzers either fail to understand custom aliases or generate false positives when macros alter the apparent syntax.

FerricCP addresses this during **Pass 1** of the analysis pipeline.

The AST walker specifically identifies `preproc_def` nodes. Whenever a `#define` is encountered, the macro definition is recorded inside the Symbol Table's macro registry.

Later, during semantic analysis, rules encountering unfamiliar types or loop constructs consult this registry to resolve the original C++ meaning before performing validation.

This allows FerricCP to analyze heavily macro driven competitive programming templates accurately without requiring preprocessing.

---

### Why the Symbol Table Matters

The Symbol Table transforms FerricCP from a syntactic linter into a genuine semantic analyzer.

Rather than simply recognizing syntax patterns, semantic rules can answer questions such as:

- Has this variable been initialized?
- Which declaration does this identifier actually refer to?
- Has this identifier been shadowed?
- What is the resolved type after macro expansion?
- Does the current scope permit this access?

By combining a language agnostic Symbol Table, strongly typed configuration, composite identifier fingerprinting, and macro aware state tracking, FerricCP performs accurate semantic reasoning while maintaining the sub-millisecond execution times required for competitive programming workflows.

## 5. The Two-Pass Analysis Pipeline

FerricCP combines all architectural components into a decoupled **two-pass compilation pipeline**.

---

### Pass 1: Global State Collection

The first pass performs a complete AST traversal to establish historical program state.

During this stage, FerricCP:

- Identifies variable declarations.
- Detects macro definitions (`#define`).
- Records assignments and initialization states.
- Builds the language agnostic `SymbolTable`.
- Generates unique variable fingerprints using:

```text
{name}_{line}
```

This pass establishes all information required for later semantic analysis.

---

### Pass 2: Hybrid Query Execution & Routing

During the second pass, FerricCP executes every precompiled Tree-sitter query.

Whenever the C backend yields a matching AST node, execution branches according to the rule type.

#### Path A: Syntactic Rules

If the YAML rule specifies:

```yaml
rule_type: syntactic
```

the violation requires no historical program state.

Examples include:

- usage of `goto`
- banned language constructs
- formatting or style violations

The engine:

1. Evaluates the match immediately.
2. Reads the diagnostic message directly from YAML.
3. Append to the vector of all violations found so far.

No Rust rule implementation is required.

---

#### Path B: Semantic Rules

If the YAML specifies:

```yaml
rule_type: semantic
```

the node requires contextual memory analysis.

The execution flow is:

1. The matching node is forwarded to the `RuleDispatcher`.
2. The dispatcher identifies the corresponding Rust implementation in `rules.rs`.
3. The rule's trait method performs an **O(1)** lookup against the pre-built `SymbolTable`.
4. If the semantic condition fails (for example, detecting an uninitialized variable), the rule similarly appends to the vector of violations.

This separation allows inexpensive syntactic rules to remain fully data driven while reserving Rust implementations only for analyses requiring historical program state.

---

### Overall Pipeline

```text
           Source Code
                 │
                 ▼
        Tree-sitter Parser
                 │
                 ▼
        Concrete Syntax Tree
                 │
        ┌────────┴────────┐
        │                 │
        ▼                 ▼
 Pass 1: State      Compile YAML Queries
 Collection          (Boot Time)
        │                 │
        └────────┬────────┘
                 ▼
      Pass 2: Query Execution
                 │
         Matching AST Nodes
                 │
        ┌────────┴─────────┐
        ▼                  ▼
 Syntactic Rule      Semantic Rule
      │                    │
      ▼                    ▼
 Immediate         RuleDispatcher
 Diagnostic               │
                          ▼
                 Symbol Table Lookup
                           │
                           ▼
                     Diagnostic Output
```

## 6. The Presentation Layer

One of the most common architectural mistakes in beginner static analyzers is coupling bug detection directly to console output. As soon as a rule detects a violation, it immediately calls `println!()` (or an equivalent logging function).

While simple, this tightly couples the analysis engine to a specific user interface, making the analyzer difficult to integrate into editors, automated tooling, or other applications.

FerricCP avoids this entirely by **strictly separating the Analysis Layer from the Presentation Layer** using the **Data Transfer Object (DTO)** pattern, implemented in `diagnostics.rs`.

---

### The `Diagnostic` DTO

Neither the `RuleDispatcher` nor the syntactic analysis engine ever writes directly to stdout.

Instead, every detected violation is packaged into a standardized `Diagnostic` object.

```rust
pub struct Diagnostic {
    pub line: usize,
    pub column: usize,
    pub id: String,
    pub message: String,
    pub severity: String,
    pub tip: Option<String>,
    pub snippet: String,
}
```

This structure acts as a universal transport mechanism between the analysis engine and any presentation layer.

Each `Diagnostic` contains:

- `line` and `column` - the exact source location.
- `id` - the unique rule identifier from the YAML configuration.
- `message` - the human readable diagnostic message.
- `severity` - the classification (Error, Warning, Info, etc.).
- `tip` - an optional remediation suggestion.
- `snippet` - the relevant slice of source code.

Because every rule emits the same standardized object, the rest of the application never needs to understand how a particular violation was detected.

---

### Diagnostic Flow

```text
              AST Match Found
                     │
                     ▼
      Syntactic Rule / RuleDispatcher
                     │
                     ▼
        Semantic Validation Executes
                     │
                     ▼
        Construct Diagnostic DTO
                     │
                     ▼
      Push into Vec<Diagnostic>
                     │
                     ▼
         Analysis Phase Complete
                     │
                     ▼
      Presentation Layer Consumes DTOs
                     │
          ┌──────────┼──────────┐
          ▼          ▼          ▼
     Terminal      JSON        LSP
      Output      Export     Integration
```

---

### Why This Architecture Matters

Returning structured `Diagnostic` objects instead of formatted text completely decouples FerricCP's compiler logic from its user interface.

This design enables several important extensions without modifying the analysis engine.

#### Language Server Protocol (LSP)

Since diagnostics already exist as structured data with source locations, FerricCP can be wrapped inside a Language Server Protocol implementation.

The same diagnostics can be streamed directly into editors such as **VS Code**, **NeoVim**, or any other IDE supporting LSP, enabling real time error highlighting without changing a single semantic rule.

---

#### CI/CD Integration

Because diagnostics are ordinary Rust structures, the CLI can serialize them directly into JSON.

This allows tools such as GitHub Actions, automated contest pipelines, or custom grading systems to consume FerricCP programmatically instead of scraping terminal output.

Example:

```json
{
    "line": 26,
    "column": 9,
    "id": "ban_goto",
    "message": "Use of 'goto' statement is banned.",
    "severity": "critical",
    "tip": "Refactor this control flow using loops or early returns.",
    "snippet": "goto loop_start;"
  }
```

---

#### Presentation Agnosticism

The analysis engine has no knowledge of:

- ANSI terminal colors
- Pretty print formatting
- Text wrapping
- Markdown rendering
- IDE diagnostics
- JSON serialization

Its only responsibility is producing mathematically correct semantic results.

The CLI is solely responsible for deciding how those diagnostics should be presented to the user.

This separation of concerns keeps the compiler core lightweight, testable, and completely independent of its execution environment.

---

### Overall Architecture

```text
          Source Code
                │
                ▼
      Hybrid Analysis Engine
                │
                ▼
     Vec<Diagnostic> (DTO Layer)
                │
      ┌─────────┼─────────┐
      ▼         ▼         ▼
  CLI Output  JSON API   LSP Server
      │         │         │
      ▼         ▼         ▼
   Terminal   CI/CD     IDE Editor
```

By treating diagnostics as standardized data rather than formatted text, FerricCP cleanly separates analysis from presentation. This makes the core engine reusable across CLIs, IDEs, APIs, and any future integrations without requiring changes to the semantic analysis pipeline.


### Summary

By combining:

- a language agnostic AST abstraction,
- Tree-sitter query based filtering,
- modular YAML rule definitions,
- trait based polymorphism,
- an optimized Symbol Table,
- and a two-pass semantic analysis pipeline,

FerricCP is able to detect many of the most common competitive programming pitfalls in milliseconds often preventing Wrong Answers before code submission.

## The Entire Architecture
```text
                    Source Code
                         │
                         ▼
                 Tree-sitter Parser
                         │
                         ▼
                Concrete Syntax Tree
                         │
          ┌──────────────┴──────────────┐
          ▼                             ▼
  Pass 1: Symbol Collection      Query Compilation
          │                             │
          └──────────────┬──────────────┘
                         ▼
                 Query Execution
                         │
                Matching AST Nodes
                         │
          ┌──────────────┴──────────────┐
          ▼                             ▼
   Syntactic Rules             Semantic Rules
                                       │
                                       ▼
                                Symbol Table
                                       │
                                       ▼
                               Diagnostic DTO
                                       │
                                       ▼
                               CLI / JSON / LSP
```