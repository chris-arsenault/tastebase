import { useEffect, useRef, useState } from "react";

const waitForVideoReady = (video: HTMLVideoElement) =>
  new Promise<void>((resolve) => {
    if (video.readyState >= 2 && video.videoWidth > 0) {
      resolve();
      return;
    }
    const handleReady = () => resolve();
    video.addEventListener("loadedmetadata", handleReady, { once: true });
    video.addEventListener("canplay", handleReady, { once: true });
  });

const getStreamDimensions = (stream: MediaStream | null) => {
  const settings = stream?.getVideoTracks()[0]?.getSettings?.();
  return { width: settings?.width, height: settings?.height };
};

const getVideoDimensions = (
  video: HTMLVideoElement,
  stream: MediaStream | null,
) => {
  const streamDims = getStreamDimensions(stream);
  const width = video.videoWidth || streamDims.width || 640;
  const height = video.videoHeight || streamDims.height || 480;
  return { width, height };
};

const captureFrame = (
  video: HTMLVideoElement,
  stream: MediaStream | null,
): string | null => {
  const canvas = document.createElement("canvas");
  const { width, height } = getVideoDimensions(video, stream);
  canvas.width = width;
  canvas.height = height;
  const context = canvas.getContext("2d");
  if (!context) return null;
  context.drawImage(video, 0, 0, canvas.width, canvas.height);
  const dataUrl = canvas.toDataURL("image/jpeg", 0.9);
  const payload = dataUrl.split(",")[1] ?? "";
  return payload.length >= 100 ? dataUrl : null;
};

export type CameraControls = ReturnType<typeof useCamera>[1];

function useCameraStream(
  videoRef: React.RefObject<HTMLVideoElement | null>,
  stream: MediaStream | null,
  active: boolean,
  onError: (msg: string) => void,
) {
  useEffect(() => {
    const video = videoRef.current;
    if (!stream || !active || !video) return;
    video.srcObject = stream;
    video.play().catch(() => onError("Unable to start camera preview."));
    return () => {
      if (video.srcObject === stream) video.srcObject = null;
    };
  }, [stream, active, onError, videoRef]);

  useEffect(() => {
    return () => stream?.getTracks().forEach((track) => track.stop());
  }, [stream]);
}

export function useCamera(onError: (msg: string) => void) {
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const [active, setActive] = useState(false);
  const [stream, setStream] = useState<MediaStream | null>(null);
  const [preview, setPreview] = useState("");
  const [base64, setBase64] = useState("");
  const [mimeType, setMimeType] = useState("");

  useCameraStream(videoRef, stream, active, onError);

  const stop = () => {
    stream?.getTracks().forEach((track) => track.stop());
    if (videoRef.current) videoRef.current.srcObject = null;
    setStream(null);
    setActive(false);
  };

  const start = () => {
    navigator.mediaDevices
      .getUserMedia({ video: { facingMode: "environment" } })
      .then((mediaStream) => {
        setStream(mediaStream);
        setActive(true);
      })
      .catch(() => onError("Camera access denied or unavailable."));
  };

  const capture = () => {
    if (!videoRef.current) {
      onError("Camera preview is not ready.");
      return;
    }
    const video = videoRef.current;
    waitForVideoReady(video)
      .then(() => {
        const dataUrl = captureFrame(video, stream);
        if (!dataUrl) {
          onError("Unable to capture photo. Try again.");
          return;
        }
        setPreview(dataUrl);
        setBase64(dataUrl);
        setMimeType("image/jpeg");
        stop();
      })
      .catch(() => onError("Unable to capture photo. Try again."));
  };

  const clear = () => {
    setPreview("");
    setBase64("");
    setMimeType("");
  };

  return [
    videoRef,
    {
      active,
      preview,
      base64,
      mimeType,
      start,
      stop,
      capture,
      clear,
      setExistingPreview: setPreview,
    },
  ] as const;
}
