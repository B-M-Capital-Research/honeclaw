> Preface: We started building HONE in February and went through a full rewrite from Python + Node.js to Rust. That transition gave us a lot of practical experience, and it made us believe Rust is one of the best choices for the AI Coding era. This post summarizes why we made that choice and why more teams should consider moving this decision as far left as possible.

# TLDR

1. **Context is precious:** Rust lets a codebase express more business logic with less context. When context is scarce, that saves tokens. Its async and concurrency model is also safer and more direct than stacks that rely heavily on framework conventions and boilerplate.
2. **No need for a mixed stack:** Rust can power backend services, frontend-adjacent modules, and desktop clients. Its performance is close to C++, while deployment and startup are much simpler.
3. **Memory safety, and compilation catches most technical mistakes:** Rust's ownership and type system move many defects into compile time instead of letting them repeatedly surface at runtime.
4. **Low resource usage and high stability:** Rust rewrites of Agent, CLI, and desktop tools often reduce memory usage, startup latency, and binary size by an order of magnitude.
5. **AI Coding changes the learning curve:** Rust is hard to learn, but AI can now handle much of the syntax and scaffolding. Teams should spend more attention on review, safety, and engineering judgment.

# 1. Context is precious

Anthropic Skills has become a popular idea, and many Agent systems are moving toward skill-based plugin models. The core idea behind progressive disclosure is context management: context is scarce, so it must be used deliberately. In the early MCP era, systems often pushed hundreds of tools and metadata blobs into the context window. Now the industry is moving back toward skill-based governance. All of this is Context Engineering.

Long context creates several problems. The most obvious one is compute cost, which is still expensive. The second is model quality. This can be hard to feel in small demos, but once you run large and complex projects, AI Coding becomes much less reliable in a single pass. The model either stops halfway or misses important edges, leaving humans to clean up the gaps.

Our practical experience is that Rust mitigates this problem significantly. Language and stack choices have a huge effect on engineering structure. Choosing well at the very beginning lets a project compound the benefits earlier.

Look at the pain points in Java. Java is one of the most widely used languages, and it has long encouraged one-public-class-per-file, deep package hierarchies, and design patterns. When an AI needs to understand a core business path, it often has to load dozens of interfaces, implementations, DTOs, domain objects, and configuration files into context. Even a Spring Boot Hello World can involve multiple folders and files. That is painful for Function Calling and multi-turn ReAct agents.

> Example 1: Java projects often create many folders, many files, and long names. Even a modern project like Spring AI is cleaner than many older Java systems, but it still carries deep package structure, long identifiers, and object-oriented conventions that inflate context.

![Java project structure context sprawl example](/blog/why-hone-uses-rust-java-files.png)

> Example 2: A lot of text exists purely to describe collaboration boundaries. Java developers know the DO / DTO pattern well. A simple object can easily require nearly a hundred lines and an entire file.

![Java DTO boilerplate example](/blog/why-hone-uses-rust-java-dto.png)

Rust is much more token-efficient in how it organizes code. It combines the rigor of systems programming with modern functional abstractions, so an AI can understand deeper system semantics inside a smaller context window.

> Example 1: Rust projects usually avoid excessive file names, huge naming hierarchies, and convention-heavy structure. OpenAI Codex is a good example: even where the code appears AI-assisted, it stays compact, readable, and direct, especially around async paths.

> Example 2: Model definitions, processing logic, assembly logic, and unit tests can sit close together. Qdrant is a good example of dense local organization: functions, enums, traits, error handling, and tests can stay in one locality, which helps the AI focus.

Async and concurrency are among the easiest areas for AI to get wrong. Java's traditional threading model and concurrency frameworks often involve thread-pool management, lock allocation, callback chains, and a lot of scaffolding. Humans make mistakes there; AI makes more. In Agent systems, SSE and streaming code can easily introduce deadlocks or performance problems.

Rust's async/await model, combined with ownership checks, provides zero-cost abstraction. When writing code that handles many network connections, the AI can use concise closures and `await`; the compiler expands that into an efficient state machine. The AI does not need to reason about every low-level state transition and scheduling detail in the prompt. The code stays more direct, which makes complex concurrency more tractable under limited token budgets.

# 2. One fast stack across endpoints

As software evolves, architecture complexity often grows exponentially with cross-platform requirements. Many AI tools now need a web service plus macOS and Windows clients. OpenClaw, Codex, Qwen, Seed, and similar projects all point in the same direction: AI tools increasingly need multi-endpoint support.

The traditional split was backend teams, frontend teams, and client teams each working in separate stacks. AI-native development and full-stack workflows push in the opposite direction. The stacks are converging, but the language boundary often remains. A project might use Java on the backend, JavaScript on the frontend, Swift or C# on the client, and several ecosystems around them. For human teams, that creates communication cost and hiring cost. For AI, it means learning multiple stacks, protocols, build systems, and deployment models.

Rust has an interesting advantage: it can write backend services, increasingly participates in frontend-adjacent tooling, and is strong on desktop clients. More importantly, it is fast, lightweight, and straightforward to ship.

Start with desktop. Electron has long been a performance pain point for cross-platform desktop apps because each app bundles a full Chromium engine and Node.js runtime. Even a Hello World-style app can exceed 150MB, start slowly, and consume 300MB to 500MB of memory while idle.

Tauri takes a different path. It uses the operating system's native WebView and moves expensive business logic and system API calls into a lightweight Rust host process. This can shrink Rust + Tauri apps to roughly 3MB to 15MB, reduce runtime memory, and make long-running local tools much more comfortable.

On the web side, Rust's market share is still smaller because ecosystem maturity matters. But modern frontend build tooling already relies heavily on Rust, and Rust's WASM story keeps improving. Frameworks like Dioxus and Leptos increasingly offer SSR, isomorphic rendering, and fine-grained reactivity patterns that feel closer to React and Next.js while keeping Rust's safety and performance model.

If a team is willing to be aggressive, Rust makes it easier to build a unified backend, frontend-adjacent, and desktop architecture:

1. **Reuse data models:** Core structures, validation rules, and domain logic can be written once and reused across backend logic, frontend rendering paths, and Tauri clients.
2. **Remove context fractures:** The AI no longer has to learn complex cross-language protocols. It can work inside one strong type system from database queries and middleware to UI flows.

# 3. Compilation as a test boundary

Blank frontend pages are a familiar Vibe Coding failure mode. You open the page, it is blank, the console shows a TypeError, and the only option is to paste the error back into the AI and ask it to fix the issue. Then you click around and another similar runtime error appears. That is classic runtime failure, often caused by type collapse.

![Runtime TypeError blank page example](/blog/why-hone-uses-rust-runtime-error.png)

In an AI-heavy development loop, the scariest problem is hallucination. AI can generate code that looks perfect syntactically and stylistically while being wrong in deeper semantics, resource lifetime, or concurrent state. In practice, many of these defects only appear at runtime.

Every AI-assisted feature can introduce a new runtime crash in an unknown corner. The project falls into a loop: AI generates code, runtime crashes, debugging is difficult, tests are generated after the fact, and regression repeats. A language choice cannot eliminate this completely, but it can reduce the blast radius.

Rust's advantage is that many technical runtime failures are moved into compile time. Rust's default posture is a compile-time safety harness. GitHub's article “Why AI is pushing developers toward typed languages” made related points:

1. **AI lacks the implicit contract a human has when writing code manually.** Whether a function accepts null, returns a string, or returns a buffer can easily drift.
2. **Type errors are common, and dynamic languages often reveal them only at runtime.** Rust's ownership and type system provide earlier and more precise feedback after generation.
3. **Rust is becoming a foundation for complex low-level modules.** When AI-generated Rust code compiles, it has already passed a meaningful stability boundary.
4. **AI Coding tools increasingly rely on strict type feedback.** Type errors give models one of the clearest retry signals.

The Rust community often says: if it compiles, it is already mostly right. That compile-time defense creates practical value. Google's Android experience also showed that moving new low-level code to Rust can dramatically reduce memory-safety vulnerability density. In the AI Coding era, that compounding effect becomes even more visible.

# 4. Performance and resource gains

OpenClaw became popular quickly, but many developers criticized its TypeScript implementation, rapid codebase growth, weak low-level isolation, RCE exposure, and supply-chain risk. Performance was also a concern: a single Agent could consume substantial memory and start slowly, making local deployment uncomfortable on ordinary hardware.

NEAR AI's team responded with IronClaw, a Rust rewrite focused on safety and resource efficiency.

| Core metric | OpenClaw (TS/Node architecture) | IronClaw (Rust/WASM architecture) | Improvement |
| --- | --- | --- | --- |
| Baseline memory | >1000MB | Around 5MB-7.8MB | Nearly 120x lower |
| Cold start latency | >500ms | <10ms | Nearly 50x faster |
| Core binary size | Around 28MB plus a large runtime | 3.4MB single standalone file | Extremely compact |
| Code isolation | Docker / process-level isolation with high overhead and permission-sprawl risk | Stateless WebAssembly sandbox with low overhead and capability control | Better safety and performance |
| Security boundary | Application-layer logic inside a large codebase | Small attack surface, network allowlists, layered defense | Architectural upgrade |

# 5. Move boldly at the far left

In traditional software engineering, Rust is famous for memory safety, zero-cost abstraction, and native performance. Its steep learning curve, however, has long been viewed as a blocker for fast iteration and broad open-source adoption.

AI Coding changes the economics. When AI can generate much of the basic code, scaffolding, and even complex business logic, Rust's syntax cost is greatly reduced. At the same time, the compiler's safety benefits become much more valuable.

For new open-source projects and enterprise architectures, choosing Rust early is not a language-preference argument. It is a forward-looking engineering decision. It can produce smaller and cleaner code, safer runtime behavior, code structures that fit AI long-context reasoning, a more unified cloud / edge / desktop stack, and much lower resource usage.

> Keep following HONE's iteration. We will publish more practical engineering notes from the project.
