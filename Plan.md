# Project: AssistantCabinetAI-SpeechLab
## Technical Proof of Concept — Offline Speech-to-Text & Text-to-Speech Evaluation

### 1. Role and Mission

Act as a senior software architect and speech-processing engineer specializing in cross-platform desktop applications, native audio processing, and offline AI inference.

Your mission is to design and implement an independent experimental project named **AssistantCabinetAI-SpeechLab**.

This project is a technical research and validation environment intended to evaluate open-source Speech-to-Text (STT) and Text-to-Speech (TTS) engines before their integration into AssistantCabinetAI.

The primary candidates are:

- whisper.cpp
- sherpa-onnx

The project must evaluate their technical feasibility, transcription accuracy, latency, resource consumption, language support, voice quality, and compatibility with Windows and macOS.

This is an experimental project, NOT a production application.

Do not modify the main AssistantCabinetAI repository.

---

# 2. Main Objectives

The project must answer the following questions through actual tests rather than assumptions:

1. Which STT engine provides the most reliable French transcription?
2. How do the engines perform with different French accents, particularly French and Swiss French?
3. How accurately do they recognize specialized medical, administrative, legal, and technical vocabulary?
4. How reliable is English transcription?
5. Can both engines operate locally without Internet connectivity?
6. What are their CPU, RAM, latency, and disk-space requirements?
7. Which engine is more suitable for short voice queries and which for longer dictation?
8. Can sherpa-onnx provide sufficiently natural French and English TTS?
9. Are male and female voice options available for both languages?
10. Can playback speed be adjusted?
11. Can both engines be integrated into a future Tauri 2 desktop application without tightly coupling the application to either engine?
12. What are the practical differences between Windows and macOS deployments?

The final deliverable must include a technical prototype, reproducible benchmark results, documented limitations, and a decision report.

---

# 3. Technology Strategy

Before implementation, inspect the available development environment and verify the current official documentation and compatibility of the candidate libraries.

Do not assume that a particular API, model, language, or platform is supported without verification.

### Proposed technology stack

- Tauri 2
- React
- TypeScript
- Vite
- Rust only where required by Tauri or native integration
- whisper.cpp native backend
- sherpa-onnx native backend
- ONNX Runtime where required
- Python only for optional benchmarking, dataset preparation, or evaluation utilities

Avoid unnecessary dependencies.

The developer must not be required to write a complex Rust application.

Prefer existing native libraries, supported bindings, and clearly isolated adapters.

Do not introduce a Python server as a mandatory runtime dependency unless the evaluation demonstrates that it is necessary.

### Cross-platform targets

- Windows 10/11 x64
- macOS Apple Silicon
- macOS Intel, where technically supported

Clearly distinguish verified compatibility from theoretical compatibility.

---

# 4. Architecture Requirements

Implement a modular architecture designed to support future integration into AssistantCabinetAI.

The application must not depend directly on a particular speech engine.

Create abstract interfaces such as:

- SpeechToTextProvider
- TextToSpeechProvider
- AudioCaptureService
- SpeechModelManager
- SpeechBenchmarkService

Proposed adapters:

- WhisperCppProvider
- SherpaOnnxProvider

The UI must communicate with these interfaces rather than directly invoking engine-specific APIs.

The architecture must allow additional providers to be added later without rewriting the UI.

Document the proposed integration boundary between the experimental project and AssistantCabinetAI.

Do not introduce unnecessary microservices or network dependencies.

---

# 5. Experimental Application Features

Build a simple but functional desktop interface with the following sections.

## A. Speech-to-Text Laboratory

Provide:

- Microphone recording
- Start / stop recording
- Audio file import (WAV initially)
- Audio playback
- Engine selection
- Model selection
- Language selection
- Transcription execution
- Transcription result display
- Editable transcription
- Processing duration
- Real-time or near-real-time latency measurements where supported
- Error reporting

Supported languages:

- French (fr)
- English (en)

The language must be explicitly configurable.

Test whether automatic language detection is available and reliable, but do not make it mandatory.

## B. Engine Comparison Mode

Allow the same audio recording to be processed by both engines.

Display:

- Whisper.cpp output
- sherpa-onnx output
- Processing time
- Relative latency
- Word Error Rate (WER), when a reference transcript exists
- Character Error Rate (CER)
- Differences between the transcriptions

The application must not arbitrarily declare one engine superior based on a single recording.

## C. Text-to-Speech Laboratory

Use sherpa-onnx as the primary TTS candidate.

Provide:

- Text input
- Language selection
- Available voice selection
- Voice identification and metadata
- Male/female voice options when available
- Playback
- Pause / resume / stop, where supported
- Adjustable speech rate
- Audio export to WAV
- Generation duration
- Audio playback duration

Evaluate French and English voices independently.

Do not assume that all voices support speed control or equivalent quality.

If a desired voice is unavailable, document the limitation and investigate alternative compatible open-source models.

## D. Model Management

Provide a basic inventory of available models.

For each model, record:

- Name
- Provider
- Language(s)
- Model architecture
- Model size
- License
- Source URL
- Required runtime
- Supported platform
- Expected memory requirements
- Installation status

Models must be downloaded only from verified official or trusted sources.

Do not bundle large models unnecessarily inside the application installer.

---

# 6. Speech Recognition Evaluation

This is one of the most important parts of the project.

Build a reproducible test suite.

### A. French language evaluation

Evaluate:

- Standard metropolitan French
- French with regional accents
- Swiss French / Suisse romande speech samples, where legitimately available
- Different speaking speeds
- Clear speech
- Natural conversational speech
- Short questions
- Longer dictation

Do not claim to cover all French or Swiss accents from a limited dataset.

Document the origin and limitations of each test sample.

### B. English evaluation

Evaluate:

- Standard English
- Different speaking speeds
- Short questions
- Longer dictation
- Technical terminology

### C. Specialized vocabulary

Create separate evaluation categories.

**Medical vocabulary**

Examples:

- Amoxicilline
- Paracétamol
- Anticoagulant
- Posologie
- Contre-indication
- Hypertension artérielle

**Administrative and legal vocabulary**

Examples:

- Attestation
- Convention
- Responsabilité civile
- Consentement
- Confidentialité

**IT and technical vocabulary**

Examples:

- PostgreSQL
- Kubernetes
- API REST
- Symfony
- Infrastructure
- Authentification

Include English technical terminology embedded in French sentences.

These are evaluation examples, not an exhaustive vocabulary.

### D. Accuracy metrics

Implement or use reliable tools to calculate:

- Word Error Rate (WER)
- Character Error Rate (CER)
- Processing time
- Real-time factor (RTF)
- Memory usage
- Failure rate

Preserve original and normalized transcripts separately.

Normalization must not hide clinically or technically significant errors.

In particular, preserve distinctions involving:

- Numbers
- Decimal values
- Dates
- Units
- Medication names
- Dosages
- Negations

A transcription that changes a dosage or removes a negation must be flagged as a critical semantic error, even if its overall WER is low.

---

# 7. Performance Evaluation

Measure actual performance on available hardware.

Record:

- CPU model
- RAM
- Operating system
- Architecture
- Model quantization
- Model size
- Cold-start time
- Warm-start time
- Transcription duration
- Peak memory usage
- CPU utilization where available
- TTS generation latency
- Audio duration versus processing duration

Distinguish CPU inference from GPU or accelerator inference.

Do not invent performance figures or extrapolate results between machines.

The benchmark must be repeatable.

---

# 8. Offline and Privacy Validation

A fundamental requirement of AssistantCabinetAI is local processing.

Verify that:

- Microphone audio is processed locally.
- Transcription does not require a cloud API.
- TTS does not require a cloud API.
- Model inference continues after Internet disconnection.
- No audio or transcript is transmitted externally by the application.
- Download operations are separated from runtime inference.

Document any network dependencies introduced by frameworks or model-loading mechanisms.

Do not use real patient data.

Use synthetic or appropriately licensed audio samples.

---

# 9. Tauri Integration Feasibility

Evaluate the practical integration options for both engines.

Investigate:

- Native library integration
- Rust bindings
- C/C++ FFI where appropriate
- External native processes as an alternative
- Windows packaging
- macOS packaging
- Native library distribution
- Model file management
- CPU architecture compatibility
- Application startup
- Error handling
- Cancellation of long-running inference

Do not assume that a successful standalone CLI test proves successful Tauri integration.

Build at least one functional end-to-end Tauri integration for each candidate if feasible.

Clearly document any blockers.

Avoid implementing complex features merely to demonstrate technical sophistication.

---

# 10. Licensing and Distribution

For every engine, model, runtime, and relevant dependency, investigate:

- Software license
- Model license
- Commercial use permissions
- Redistribution restrictions
- Attribution requirements
- Third-party dependencies

The ultimate objective is a potentially commercial product.

Do not assume that an open-source engine automatically makes every associated model suitable for commercial redistribution.

Produce a licensing compatibility table.

---

# 11. Benchmark Dataset and Reproducibility

Create a structured test dataset with:

- Unique test ID
- Language
- Accent / speech profile, when known
- Domain
- Audio file
- Reference transcript
- Audio duration
- Source and license
- Expected critical terms

Provide a way to add new test samples without changing application code.

All test data must be synthetic, public-domain, appropriately licensed, or explicitly authorized.

Do not store sensitive recordings.

---

# 12. Final Decision Report

Generate a report in Markdown:

`docs/SPEECH_ENGINE_EVALUATION.md`

The report must contain:

### Executive summary

A concise description of findings and unresolved questions.

### Engine comparison

Compare Whisper.cpp and sherpa-onnx based on measured evidence.

### French accuracy

Results for general French and available regional/accent test categories.

### English accuracy

Results and language-specific limitations.

### Specialized vocabulary

Detailed analysis of terminology errors and critical semantic errors.

### TTS assessment

Voice availability, language quality, naturalness, speed control, and latency.

### Performance

Measured CPU, RAM, startup time, and processing speed.

### Windows/macOS compatibility

Verified capabilities, integration difficulties, and packaging constraints.

### Privacy and offline operation

Evidence and limitations.

### Licensing

Relevant software and model licensing findings.

### Architecture recommendation

Recommend an integration strategy based on the evidence:

- Whisper.cpp as primary STT
- sherpa-onnx as primary STT
- Both engines as interchangeable providers
- Different engines for different usage scenarios
- Alternative approach if neither meets requirements

Do not force a winner if results are inconclusive.

### Integration roadmap

Propose a subsequent implementation sequence for AssistantCabinetAI.

---

# 13. Development Methodology

Work incrementally.

Before implementing the application:

1. Inspect current official documentation.
2. Verify platform support.
3. Identify viable models.
4. Identify licensing constraints.
5. Propose the architecture.
6. Present a short implementation plan.

Then implement in small, testable milestones.

Suggested milestones:

- M0: Feasibility and dependency validation
- M1: Minimal Tauri application
- M2: Whisper.cpp integration
- M3: sherpa-onnx integration
- M4: Audio capture and transcription comparison
- M5: Benchmarking and evaluation metrics
- M6: TTS experimentation
- M7: Windows/macOS packaging validation
- M8: Final report and integration recommendation

Do not silently skip a milestone.

If a platform or dependency cannot be tested in the current environment, mark it as NOT VERIFIED and provide the exact steps needed for validation.

---

# 14. Expected Deliverables

The final repository must include:

- Functional experimental Tauri application
- Modular STT/TTS architecture
- Whisper.cpp adapter
- sherpa-onnx adapter
- Microphone recording and WAV import
- Engine comparison interface
- French and English test suites
- WER/CER evaluation
- Performance benchmark results
- TTS voice evaluation
- Model inventory
- Licensing assessment
- Windows/macOS compatibility report
- Reproducible test instructions
- `docs/SPEECH_ENGINE_EVALUATION.md`
- `README.md` with installation and execution instructions

The project must remain independent from AssistantCabinetAI.

---

# 15. Important Engineering Constraints

- Prioritize correctness over visual polish.
- Do not introduce cloud dependencies.
- Do not use simulated benchmark data.
- Do not claim compatibility without evidence.
- Do not hardcode a single engine into the UI.
- Do not assume that all models support French, English, or voice selection.
- Do not automatically send recognized speech to an AI agent.
- Do not automatically play TTS unless explicitly enabled.
- Do not implement medical decision-making or clinical recommendations.
- Preserve a clear separation between experimental code and future production code.

At every milestone, summarize:

1. What has been implemented.
2. What has been verified.
3. What remains unverified.
4. Known limitations.
5. Recommended next step.

The ultimate goal is to deliver a technically credible and reproducible foundation for the future voice interaction system of AssistantCabinetAI.