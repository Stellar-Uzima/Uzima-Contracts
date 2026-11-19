# Voice Interface (Experimental)

`mobile-sdk/core/src/voice/VoiceInterface.ts` is a **reference implementation**.
Its transcription, realtime streaming and voice-biometric methods are simulated:

- `transcribe()` decodes a supplied string/ArrayBuffer rather than running a
  speech-to-text model.
- `startRealtimeTranscription()` replays a fixed list of phrase chunks.
- `authenticateVoiceBiometric()` does **not** accept phrase-based proof of
  identity; it always defers to `BiometricAuth` with a PIN fallback.

## Limitations

- No speaker verification, liveness detection or anti-replay protection.
- No streaming transport; the simulated source ignores `AudioEnvelope` frames.
- Not suitable for authentication or any HIPAA-regulated decision on its own.

## Planned stable API

The exported `AudioEnvelope` and `AudioStreamSource` types describe the
intended contract for a production implementation:

```ts
const source: AudioStreamSource = platformRecorder;
const sub = source.subscribe((frame: AudioEnvelope) => {
  // decrypt -> verify frame.checksum -> feed speech pipeline
});
await source.start();
// ...
sub.unsubscribe();
await source.stop();
```

A frame's `checksum` must be verified against the decrypted payload before the
frame is trusted. Until a real engine is wired in, callers must treat all voice
output as untrusted and keep the biometric/PIN fallback enabled.
