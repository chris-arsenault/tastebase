import { useEffect, useRef, useState } from "react";

const fileToBase64 = (blob: Blob): Promise<string> => {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(typeof reader.result === "string" ? reader.result : "");
    reader.onerror = () => reject(reader.error ?? new Error("FileReader failed"));
    reader.readAsDataURL(blob);
  });
};

export type RecorderControls = ReturnType<typeof useRecorder>;

export function useRecorder(onError: (msg: string) => void) {
  const recorderRef = useRef<MediaRecorder | null>(null);
  const [isRecording, setIsRecording] = useState(false);
  const [audioUrl, setAudioUrl] = useState("");
  const [audioBase64, setAudioBase64] = useState("");
  const [audioBlob, setAudioBlob] = useState<Blob | null>(null);
  const [audioMimeType, setAudioMimeType] = useState("");
  const [audioStream, setAudioStream] = useState<MediaStream | null>(null);

  useEffect(() => {
    return () => {
      audioStream?.getTracks().forEach((track) => track.stop());
    };
  }, [audioStream]);

  const onRecordingDone = (chunks: Blob[], mimeType: string, stream: MediaStream) => {
    const blob = new Blob(chunks, { type: mimeType });
    setAudioBlob(blob);
    setAudioMimeType(mimeType || "audio/webm");
    setAudioUrl(URL.createObjectURL(blob));
    fileToBase64(blob)
      .then((b64) => setAudioBase64(b64))
      .catch(() => onError("Failed to encode audio."));
    stream.getTracks().forEach((track) => track.stop());
    setAudioStream(null);
  };

  const start = () => {
    navigator.mediaDevices
      .getUserMedia({ audio: true })
      .then((stream) => {
        const recorder = new MediaRecorder(stream);
        const chunks: Blob[] = [];
        recorder.ondataavailable = (event) => {
          if (event.data.size > 0) chunks.push(event.data);
        };
        recorder.onstop = () => onRecordingDone(chunks, recorder.mimeType, stream);
        recorder.start();
        recorderRef.current = recorder;
        setAudioStream(stream);
        setIsRecording(true);
      })
      .catch(() => {
        onError("Microphone access denied or unavailable.");
      });
  };

  const stop = () => {
    recorderRef.current?.stop();
    setIsRecording(false);
  };

  const clear = () => {
    setAudioUrl("");
    setAudioBase64("");
    setAudioBlob(null);
    setAudioMimeType("");
  };

  return { isRecording, audioUrl, audioBase64, audioBlob, audioMimeType, start, stop, clear };
}
