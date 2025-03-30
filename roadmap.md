# Axo Development Roadmap

## Current Focus (Q3 2024)
```mermaid
gantt
    title Axo Development Timeline
    dateFormat  YYYY-MM-DD
    section Core Language
    Lexer/Parser Design       :active, des1, 2024-07-01, 30d
    Type System Specification :         des2, after des1, 30d
    Memory Model              :         des3, after des2, 45d

    section Tooling
    Basic CLI Implementation  :         des4, 2024-08-01, 30d
    VS Code Extension         :         des5, after des4, 45d