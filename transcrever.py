import os
import sys
import wave
import numpy as np

# Pipes em UTF-8 (no Windows o padrão é cp1252 e quebra acentos)
sys.stdin.reconfigure(encoding="utf-8")
sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

from faster_whisper import WhisperModel

MODEL_SIZE = os.environ.get("ASTER_WHISPER_MODEL", "base").strip() or "base"
CPU_THREADS = max(2, min(8, (os.cpu_count() or 4)))

model = WhisperModel(
    MODEL_SIZE,
    device="cpu",
    compute_type="int8",
    cpu_threads=CPU_THREADS,
)
print("READY", flush=True)

# Alucinações típicas do Whisper em ruído/silêncio
LIXO = (
    "amara.org",
    "legendas",
    "legendado por",
    "inscreva",
    "obrigado por assistir",
    "subtitles by",
)

# Frase natural ajuda o modelo a acertar nomes sem vazar lista seca
VOCAB_PROMPT = (
    "Comandos para a assistente Aster: abrir Spotify, Discord, WhatsApp, "
    "Brave, Vivaldi, VSCode e Steam. Usuários: Kaida e Kio."
)

PROMPT_ECHO_MARKERS = (
    "comandos para a assistente aster",
    "brave, vivaldi, vscode e steam",
    "usuários: kaida e kio",
)

VAD_AVAILABLE = True


def limpar_linha(s: str) -> str:
    return " ".join(s.split())


def carregar_wav_float32(wav_path: str) -> np.ndarray:
    """Lê o WAV 16kHz PCM direto com wave+numpy (evita bug de versão do PyAV/metadata_errors)."""
    with wave.open(wav_path, "rb") as wf:
        channels = wf.getnchannels()
        sampwidth = wf.getsampwidth()
        frames = wf.readframes(wf.getnframes())
    if sampwidth != 2:
        raise ValueError(f"WAV deve ser 16-bit PCM (recebido {sampwidth * 8}-bit)")
    audio = np.frombuffer(frames, dtype=np.int16).astype(np.float32) / 32768.0
    if channels > 1:
        audio = audio.reshape(-1, channels).mean(axis=1)
    return audio


def transcrever(wav_path: str, use_vad: bool) -> str:
    audio = carregar_wav_float32(wav_path)
    if audio.size == 0:
        return ""

    kwargs = dict(
        language="pt",
        beam_size=1,
        temperature=0.0,
        condition_on_previous_text=False,
        no_speech_threshold=0.6,
        log_prob_threshold=-0.8,
        initial_prompt=VOCAB_PROMPT,
    )
    if use_vad:
        kwargs["vad_filter"] = True
        kwargs["vad_parameters"] = dict(min_silence_duration_ms=420)

    segments, _ = model.transcribe(audio, **kwargs)
    partes = [s.text for s in segments]
    return limpar_linha(" ".join(partes))


for line in sys.stdin:
    wav_path = line.strip()
    if not wav_path:
        continue
    try:
        if not os.path.exists(wav_path):
            print("ERROR:arquivo wav nao encontrado", flush=True)
            continue

        if VAD_AVAILABLE:
            try:
                text = transcrever(wav_path, use_vad=True)
            except Exception as e:
                err_low = str(e).lower()
                if "onnx" in err_low or "silero" in err_low or "vad" in err_low:
                    VAD_AVAILABLE = False
                    print(f"vad indisponivel, desativando: {e}", file=sys.stderr, flush=True)
                    text = transcrever(wav_path, use_vad=False)
                else:
                    raise
        else:
            text = transcrever(wav_path, use_vad=False)

        lower = text.lower()
        if (
            any(l in lower for l in LIXO)
            or any(m in lower for m in PROMPT_ECHO_MARKERS)
            or not any(c.isalnum() for c in text)
        ):
            text = ""

        print(f"RESULT:{text}", flush=True)
    except Exception as e:
        print(f"ERROR:{limpar_linha(str(e))}", flush=True)
