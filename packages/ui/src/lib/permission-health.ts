export const SCREEN_RECORDING_USED = false;
export const LAUNCH_LOOP_PROMPTING = false;

export type PermissionCapability =
  | "inputMonitoring"
  | "accessibility"
  | "launchAtLogin"
  | "automation"
  | "screenRecording"
  | "selfTest";

export type PermissionUsage = "required" | "optional" | "absentP0" | "notUsed";

export type PermissionHealthRow = {
  capability: PermissionCapability;
  state: string;
  usage: PermissionUsage;
  whyKey: string;
  retestKey: string;
  alternativeKey: string;
};

export function permissionHealthRows(states: {
  inputMonitoring: string;
  accessibility: string;
  selfTest: string;
}): PermissionHealthRow[] {
  return [
    {
      capability: "inputMonitoring",
      state: states.inputMonitoring,
      usage: "required",
      whyKey: "settings.permission.inputMonitoring.why",
      retestKey: "settings.permission.retest",
      alternativeKey: "settings.permission.inputMonitoring.alternative",
    },
    {
      capability: "accessibility",
      state: states.accessibility,
      usage: "required",
      whyKey: "settings.permission.accessibility.why",
      retestKey: "settings.permission.retest",
      alternativeKey: "settings.permission.accessibility.alternative",
    },
    {
      capability: "launchAtLogin",
      state: "not_requested",
      usage: "optional",
      whyKey: "settings.permission.launchAtLogin.why",
      retestKey: "settings.permission.retest",
      alternativeKey: "settings.permission.launchAtLogin.alternative",
    },
    {
      capability: "automation",
      state: "unavailable",
      usage: "absentP0",
      whyKey: "settings.permission.automation.why",
      retestKey: "settings.permission.retest",
      alternativeKey: "settings.permission.automation.alternative",
    },
    {
      capability: "screenRecording",
      state: "unavailable",
      usage: "notUsed",
      whyKey: "settings.permission.screenRecording.why",
      retestKey: "settings.permission.retest",
      alternativeKey: "settings.permission.screenRecording.alternative",
    },
    {
      capability: "selfTest",
      state: states.selfTest,
      usage: "required",
      whyKey: "settings.permission.selfTest.why",
      retestKey: "settings.permission.retest",
      alternativeKey: "settings.permission.selfTest.alternative",
    },
  ];
}

export function manualComposerAvailable(): boolean {
  return true;
}
