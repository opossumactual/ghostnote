// Recording State
type RecordingStatus = "idle" | "recording" | "processing";

let status = $state<RecordingStatus>("idle");
let duration = $state(0);

let intervalId: ReturnType<typeof setInterval>;

// Actions
function startRecording() {
  status = "recording";
  duration = 0;

  // Start duration counter
  intervalId = setInterval(() => {
    duration++;
  }, 1000);

  // TODO: invoke start_recording
  console.log("Recording started");
}

async function stopRecording(): Promise<string | null> {
  clearInterval(intervalId);
  status = "processing";

  // TODO: invoke stop_recording
  console.log("Processing recording...");

  // Simulate transcription delay
  await new Promise((resolve) => setTimeout(resolve, 1500));

  const transcription = "[Transcribed text would appear here]";

  status = "idle";
  duration = 0;

  return transcription;
}

function cancelRecording() {
  clearInterval(intervalId);
  status = "idle";
  duration = 0;
}

// Format duration as MM:SS
function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

// Export reactive getters and actions
export const recordingStore = {
  get status() {
    return status;
  },
  get duration() {
    return duration;
  },
  get formattedDuration() {
    return formatDuration(duration);
  },
  get isRecording() {
    return status === "recording";
  },
  get isProcessing() {
    return status === "processing";
  },
  startRecording,
  stopRecording,
  cancelRecording,
};
