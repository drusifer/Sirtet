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
    Note right of Oracle: "Sprint 8 groomed:<br/>CHAT.md archived to chat<br/>_archive/CHAT_sprint8.md<br/>, task.md checkboxes<br/>updated, DECISIONS.md<br/>records the US-39<br/>scoping corr…"
    Smith->>All: "resume"
    Note right of Smith: "Resuming as Smith.<br/>Sprint 8 end-to-end<br/>test."
    Smith->>User: "user blocked"
    Note right of Smith: "Static/HCI review: zero<br/>user-facing changes<br/>anywhere<br/>(web/index.html, menu<br/>text/bindings all<br/>untouched) - strong<br/>evidence this refactor<br/>is…"
    Note over Oracle,User: 📅 2026-08-12
    User->>All: "request"
    Note right of User: "wdyt from a ux<br/>perspective"
    Smith->>All: "resume"
    Note right of Smith: "Resuming as Smith.<br/>Reviewing the diagram<br/>redesign from a UX<br/>perspective."
    Smith->>All: "user consult"
    Note right of Smith: "Found a real, code-<br/>verified issue with the<br/>Note-based design: Note<br/>over A,B spans the full<br/>x-distance between A and<br/>B regardless of partici…"
```
