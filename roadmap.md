# Axo Development Roadmap

## Overview
Axo is a programming language in development, following the **River Flow Parsing System**. This roadmap outlines the planned stages, focusing on parsing, analysis, and tooling, adapted to fit a solo developer's workflow.

---

## Current Focus (Q2-Q3 2025)
```mermaid
gantt
title Axo Development Timeline
dateFormat  YYYY-MM-DD
section Core Language
Lexer                     :active, des1, 2025-03-01, 60d
Stream Parser (Token to Structured Flow AST) : active, des2, 2025-03-20, 30d
Semantic Analysis         :active, des3, 2025-03-31, 30d

Sema
Type System Specification :         des3, after des1, 30d
Memory Model              :         des3, after des2, 45d

section Tooling
Basic CLI Implementation  :         des4, 2025-08-01, 30d
VS Code Extension         :         des5, after des4, 45d
```

---

## Development Stages (River Flow System)

### 🌊 **Surface Form (Lexing & Tokenization)** (March–April 2025)
- **Goal**: Convert raw text into meaningful tokens.
- **Tasks**:
    - Implement tokenization rules.
    - Support basic syntax structures.
    - Create an error-handling system for lexing.

### 🌊 **Stream Form (Parsing & AST Construction)** (April–May 2025)
- **Goal**: Convert tokens into an Abstract Syntax Tree (AST).
- **Tasks**:
    - Build a structured parsing system.
    - Ensure syntactic correctness.
    - Integrate error reporting.

### 🌊 **Wave Form (Semantic Analysis & Type System)** (May–June 2025)
- **Goal**: Ensure logical consistency and enforce types.
- **Tasks**:
    - Implement a basic type system.
    - Perform variable and function name resolution.
    - Ensure memory safety rules.

### 🌊 **Deep Form (Optimization & Memory Model)** (June–July 2025)
- **Goal**: Optimize execution and define memory management.
- **Tasks**:
    - Optimize the IR for performance.
    - Define stack/heap memory rules.
    - Implement register allocation.

### 🌊 **Sand Form (Code Generation & Execution)** (July–August 2025)
- **Goal**: Translate optimized IR into executable code.
- **Tasks**:
    - Implement machine code generation.
    - Create a virtual machine or interpreter.
    - Perform final performance tests.

---

## Tooling & Ecosystem

### 🌊 **Tide Form (Debugging & CLI)** (August–September 2025)
- **Goal**: Build debugging and command-line tools.
- **Tasks**:
    - Implement a basic CLI for compilation.
    - Add logging and error tracking.

### 🌊 **Reef Form (VS Code Extension)** (September–November 2025)
- **Goal**: Provide an IDE experience for Axo.
- **Tasks**:
    - Develop syntax highlighting and autocompletion.
    - Integrate with the compiler and error checker.

---

## Summary
Axo's development follows a structured flow, from lexing to final execution, using the **River Flow Parsing System**. As a solo developer, the roadmap prioritizes essential features first, with optimizations and tooling following progressively. By the end of **2025**, the goal is to have a working prototype with basic tooling support.

