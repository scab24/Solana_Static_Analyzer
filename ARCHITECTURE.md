# Solana Analyzer - System Architecture

## System Overview

```mermaid
flowchart TD
    %% Input Layer
    USER[User] --> CLI[CLI Interface]
    CLI --> ARGS{Parse Arguments}
    
    %% Core Processing Layer
    ARGS --> ANALYZER[Core Analyzer]
    ANALYZER --> PARSER[AST Parser]
    PARSER --> AST[(Abstract Syntax Tree)]
    
    %% Modular Rule Engine
    ANALYZER --> ENGINE[Rule Engine]
    ENGINE --> HIGH_MODS[High Severity Modules]
    ENGINE --> MED_MODS[Medium Severity Modules] 
    ENGINE --> LOW_MODS[Low Severity Modules]
    
    %% Example Rule Modules (one per category)
    HIGH_MODS --> H_EXAMPLE[Rule Module Example]
    MED_MODS --> M_EXAMPLE[Rule Module Example]
    LOW_MODS --> L_EXAMPLE[Rule Module Example]
    
    H_EXAMPLE --> H_RULE[mod.rs]
    H_EXAMPLE --> H_FILTER[filters.rs]
    M_EXAMPLE --> M_RULE[mod.rs]
    M_EXAMPLE --> M_FILTER[filters.rs]
    L_EXAMPLE --> L_RULE[mod.rs]
    L_EXAMPLE --> L_FILTER[filters.rs]
    
    %% DSL Processing
    H_RULE --> DSL_CORE[DSL Core]
    M_RULE --> DSL_CORE
    L_RULE --> DSL_CORE
    
    DSL_CORE --> QUERY_ENGINE[Query Engine]
    DSL_CORE --> GENERIC_FILTERS[Generic Filters]
    
    %% Analysis Components
    QUERY_ENGINE --> BASIC_OPS[Basic Operations]
    GENERIC_FILTERS --> ANCHOR_OPS[Anchor Operations]
    
    BASIC_OPS --> SPAN_EXTRACTOR[Span Extractor]
    ANCHOR_OPS --> SPAN_EXTRACTOR
    H_FILTER --> SPAN_EXTRACTOR
    M_FILTER --> SPAN_EXTRACTOR
    L_FILTER --> SPAN_EXTRACTOR
    
    %% Results Processing
    SPAN_EXTRACTOR --> FINDINGS[Raw Findings]
    FINDINGS --> POST_PROCESSOR[Location Improver]
    POST_PROCESSOR --> ENHANCED_FINDINGS[Enhanced Findings]
    
    %% Output Layer
    ENHANCED_FINDINGS --> REPORTER[Report Generator]
    REPORTER --> OUTPUT[Professional Report]
    OUTPUT --> USER
    
    AST --> SPAN_EXTRACTOR
```

## Execution Flow

```mermaid
sequenceDiagram
    participant U as User
    participant C as CLI
    participant A as Analyzer
    participant E as Engine
    participant R as Rules
    participant D as DSL
    participant S as SpanExtractor
    participant Rep as Reporter
    
    U->>C: Execute Command
    C->>A: Initialize Analysis
    A->>A: Parse Source Files
    A->>E: Load Rule Engine
    E->>R: Register Rules
    
    loop Process Each Rule
        R->>D: Execute DSL Query
        D->>D: Apply Filters
        D->>S: Extract Locations
        S-->>D: Return Spans
        D-->>R: Return Findings
    end
    
    R-->>E: Consolidated Results
    E-->>A: Analysis Complete
    A->>Rep: Generate Report
    Rep-->>U: Output Report
```

## DSL Processing Pipeline

```mermaid
flowchart LR
    %% Input
    AST[AST Input] --> QUERY[Query Engine]
    
    %% Basic Processing
    QUERY --> BASIC[Basic Filters]
    BASIC --> FUNCTIONS[functions]
    BASIC --> STRUCTS[structs]
    BASIC --> CALLS[calls_to]
    
    %% Solana Processing
    QUERY --> SOLANA[Solana Filters]
    SOLANA --> PUBLIC[public_functions]
    SOLANA --> ACCOUNTS[derives_accounts]
    SOLANA --> UNSAFE[has_unsafe_divisions]
    
    %% Convergence
    FUNCTIONS --> PROCESSOR[Result Processor]
    STRUCTS --> PROCESSOR
    CALLS --> PROCESSOR
    PUBLIC --> PROCESSOR
    ACCOUNTS --> PROCESSOR
    UNSAFE --> PROCESSOR
    
    %% Output
    PROCESSOR --> FINDINGS[Findings]
```

## Rule Creation Workflow

```mermaid
flowchart TD
    %% Rule Definition
    START[Start Rule Creation] --> BUILDER[RuleBuilder]
    
    %% Configuration
    BUILDER --> CONFIG[Rule Configuration]
    CONFIG --> ID[Set ID]
    CONFIG --> TITLE[Set Title]
    CONFIG --> DESC[Set Description]
    CONFIG --> SEV[Set Severity]
    
    %% DSL Query
    CONFIG --> DSL[DSL Query Definition]
    DSL --> CHAIN[Filter Chain]
    
    %% Filter Types
    CHAIN --> BASIC_F[Basic Filters]
    CHAIN --> SOLANA_F[Solana Filters]
    
    %% Processing
    BASIC_F --> PROCESS[Process Results]
    SOLANA_F --> PROCESS
    PROCESS --> CONVERT[Convert to Findings]
    
    %% Output
    CONVERT --> RULE[Compiled Rule]
    RULE --> REGISTRY[Rule Registry]
```

## Data Processing Flow

```mermaid
flowchart TD
    %% Input Processing
    SOURCE[Source Code] --> PARSER[Parser]
    PARSER --> AST[AST]
    
    %% Node Extraction
    AST --> EXTRACTOR[Node Extractor]
    EXTRACTOR --> FUNCTIONS[Function Nodes]
    EXTRACTOR --> STRUCTS[Struct Nodes]
    EXTRACTOR --> IMPLS[Impl Nodes]
    
    %% Analysis
    FUNCTIONS --> ANALYZER[Analyzer]
    STRUCTS --> ANALYZER
    IMPLS --> ANALYZER
    
    %% Span Processing
    ANALYZER --> SPAN_PROC[Span Processor]
    SPAN_PROC --> LOCATIONS[Locations]
    SPAN_PROC --> SNIPPETS[Code Snippets]
    
    %% Finding Generation
    LOCATIONS --> FINDINGS[Findings]
    SNIPPETS --> FINDINGS
    ANALYZER --> FINDINGS
    
    %% Output
    FINDINGS --> FORMATTER[Report Formatter]
    FORMATTER --> REPORT[Final Report]
```

## Filter Architecture: Generic vs Rule-Specific

The modular architecture separates filters into **generic DSL filters** (reusable) and **rule-specific filters** (encapsulated):

```mermaid
flowchart TD
    %% Filter Architecture
    FILTERS[Filter System] --> GENERIC[Generic DSL Filters]
    FILTERS --> SPECIFIC[Rule-Specific Filters]
    
    %% Generic DSL Filters (Reusable)
    GENERIC --> G1[functions]
    GENERIC --> G2[structs]
    GENERIC --> G3[derives_accounts]
    GENERIC --> G4[public_functions]
    GENERIC --> G5[calls_to]
    GENERIC --> G6[filter]
    
    %% Rule-Specific Filters (Encapsulated)
    SPECIFIC --> HIGH_EXAMPLE[High Severity Example]
    SPECIFIC --> MED_EXAMPLE[Medium Severity Example]
    SPECIFIC --> LOW_EXAMPLE[Low Severity Example]
    
    %% Technology Choice per Rule
    HIGH_EXAMPLE --> ANCHOR_SYN[anchor-syn for semantic analysis]
    MED_EXAMPLE --> SYN[syn for AST analysis]
    LOW_EXAMPLE --> SYN_ALT[syn for AST analysis]
    
    %% Generic filters are shared
    G1 --> SHARED[Shared across all rules]
    G2 --> SHARED
    G3 --> SHARED
    G4 --> SHARED
    G5 --> SHARED
    G6 --> SHARED
```