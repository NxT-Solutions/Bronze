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

export const USED_PERMISSION_COMMANDS = {
  retest: "retest_used_permissions",
  openSettings: "open_privacy_settings",
} as const;

export type PermissionPromptResult = {
  input_monitoring: string;
  accessibility: string;
  listen_requested: boolean;
  accessibility_requested: boolean;
  screen_recording_requested: boolean;
};

export function isUsedPermission(capability: PermissionCapability): boolean {
  return capability === "inputMonitoring" || capability === "accessibility";
}

export function promptStateForCapability(
  capability: PermissionCapability,
  result: PermissionPromptResult,
): string | null {
  if (capability === "inputMonitoring") {
    return result.input_monitoring;
  }
  if (capability === "accessibility") {
    return result.accessibility;
  }
  return null;
}

export function shouldRevealSystemSettings(
  capability: PermissionCapability,
  result: PermissionPromptResult,
): boolean {
  if (!isUsedPermission(capability)) {
    return false;
  }
  if (result.screen_recording_requested) {
    return false;
  }
  const state = promptStateForCapability(capability, result);
  return state !== "granted_unverified" && state !== "healthy";
}

export async function retestUsedPermission(
  capability: PermissionCapability,
  invokeFn: (cmd: string) => Promise<PermissionPromptResult>,
): Promise<{ result: PermissionPromptResult; revealSettings: boolean }> {
  if (!isUsedPermission(capability)) {
    throw new Error("capability_not_used");
  }
  const result = await invokeFn(USED_PERMISSION_COMMANDS.retest);
  return {
    result,
    revealSettings: shouldRevealSystemSettings(capability, result),
  };
}
