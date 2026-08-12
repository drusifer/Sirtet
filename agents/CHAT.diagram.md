# CHAT.md — Conversation Flow

Auto-generated from `agents/CHAT.md` by `bobp chat-diagram`. Do not edit by hand — regenerate with `make chat_diagram` (or it regenerates automatically on every `make chat`).

```mermaid
%%{init: {"sequence": {"messageFontSize": 14, "noteFontSize": 18, "actorFontSize": 14}}}%%
sequenceDiagram
    autonumber
    participant Oracle
    participant Smith
    participant All
    participant User
    Note over Oracle,User: 📅 2026-08-11
    Oracle->>Smith: "ora handoff"
    Note over Oracle,Smith: "Sprint 8 groomed:<br/>CHAT.md archived to chat<br/>_archive/CHAT_sprint8.md<br/>, task.md checkboxes<br/>updated, DECISIONS.md<br/>records the US-39<br/>scoping corr…"
    Smith->>All: "resume"
    Note over Smith,All: "Resuming as Smith.<br/>Sprint 8 end-to-end<br/>test."
    Smith->>User: "user blocked"
    Note over Smith,User: "Static/HCI review: zero<br/>user-facing changes<br/>anywhere<br/>(web/index.html, menu<br/>text/bindings all<br/>untouched) - strong<br/>evidence this refactor<br/>is…"
```
