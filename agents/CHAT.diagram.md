# CHAT.md — Conversation Flow

Auto-generated from `agents/CHAT.md` by `bobp chat-diagram`. Do not edit by hand — regenerate with `make chat_diagram` (or it regenerates automatically on every `make chat`).

```mermaid
%%{init: {"themeVariables": {"fontSize": "20px"}}}%%
sequenceDiagram
    autonumber
    participant Oracle
    participant Smith
    participant All
    participant User
    Note over Oracle,User: 📅 2026-08-11
    Oracle->>Smith: "ora handoff — Sprint 8 groomed:<br/>CHAT.md archived<br/>to chat_archive/CH<br/>AT_sprint8.md,<br/>task.md checkboxes<br/>updated,<br/>DECISIONS.md<br/>records the US-39<br/>scoping corr…"
    Smith->>All: "resume — Resuming as Smith.<br/>Sprint 8 end-to-<br/>end test."
    Smith->>User: "user blocked — Static/HCI review:<br/>zero user-facing<br/>changes anywhere<br/>(web/index.html,<br/>menu text/bindings<br/>all untouched) -<br/>strong evidence<br/>this refactor is…"
```
