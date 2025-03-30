# Axo Development Roadmap

## Overview
Axo is a programming language project following the **River Flow Parsing System** for structured, optimized, and efficient code processing. As a solo developer, this roadmap will guide the structured development of Axo from parsing to execution.

---

## Current Focus (Q3 2024 - Q2 2025)

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
Basic CLI Implementation  :         des4, 2024-08-01, 30d
VS Code Extension         :         des5, after des4, 45d
```

---

## Core Language Development

### 🌊 **Surface Form (Raw Code) – Lexer**
**Timeline:** March 1, 2025 - April 30, 2025  
**Goal:** Implement a robust **Lexer** to convert raw source code into tokens.  
**Tasks:**
- Define language syntax and token categories.
- Implement a streaming tokenization system.
- Support error handling for malformed tokens.

### 🌊 **Stream Form (Structured Flow) – Parser**
**Timeline:** March 20, 2025 - April 19, 2025  
**Goal:** Develop a **Stream Parser** that builds a structured **AST (Abstract Syntax Tree)** from tokenized input.  
**Tasks:**
- Implement recursive descent parsing.
- Integrate early syntax error detection.
- Lay groundwork for semantic analysis.

### 🌊 **Wave Form (Optimized & Directed Flow) – Semantic Analysis**
**Timeline:** March 31, 2025 - April 30, 2025  
**Goal:** Implement **semantic analysis** to ensure logical consistency.
**Tasks:**
- Develop a **type checker**.
- Implement **scope resolution**.
- Build **early optimization passes**.

### 🌊 **Deep Form – Type System & Memory Model**
**Type System Specification:** April 30, 2025 - May 30, 2025  
**Memory Model Implementation:** May 15, 2025 - June 30, 2025  
**Goal:** Design a strong, safe **type system** and efficient **memory model**.  
**Tasks:**
- Define primitive and composite types.
- Implement borrowing/lifetime analysis.
- Optimize memory management strategies.

---

## Tooling & Ecosystem

### 🌊 **Basic CLI Implementation**
**Timeline:** August 1, 2024 - August 31, 2024  
**Goal:** Develop a **command-line interface (CLI)** for interacting with the compiler.
**Tasks:**
- Implement `axo build`, `axo run`, and `axo check` commands.
- Provide minimal error reporting.
- Integrate with core parsing modules.

### 🌊 **VS Code Extension**
**Timeline:** September 1, 2024 - October 15, 2024  
**Goal:** Build a **VS Code extension** for syntax highlighting and basic IDE support.
**Tasks:**
- Implement syntax highlighting.
- Add error reporting in the editor.
- Support interactive debugging hooks.

---

## Next Steps
After completing the core parsing and semantic analysis stages, the focus will shift to:
- 🌊 **Wave Form (Mid-level IR & Borrow Checking)** – Q3 2025
- 🌊 **Deep Form (Final Optimizations & Code Generation)** – Q4 2025
- 🌊 **Tooling Improvements (Debugger, LSP, More Editor Support)** – Q1 2026

This roadmap keeps development structured and ensures Axo flows smoothly from raw code to a high-performance compiled language.

