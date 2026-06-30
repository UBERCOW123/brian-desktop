import {
  BrianButton,
  BrianLabel,
  BrianSelect,
} from "@ui";
import {
  ACCENT_LABELS,
  ACCENT_PALETTES,
  type AccentPalette,
  type ThemeMode,
} from "../theme/types";

type TitlebarControlsProps = {
  themeMode: ThemeMode;
  accent: AccentPalette;
  onThemeModeChange: (mode: ThemeMode) => void;
  onAccentChange: (accent: AccentPalette) => void;
  authSlot: React.ReactNode;
};

export function TitlebarControls({
  themeMode,
  accent,
  onThemeModeChange,
  onAccentChange,
  authSlot,
}: TitlebarControlsProps) {
  return (
    <div className="flex flex-wrap items-center gap-3">
      <div className="flex items-center gap-2">
        <BrianLabel htmlFor="theme-mode" className="text-xs text-[var(--text-secondary)]">
          Theme
        </BrianLabel>
        <BrianSelect
          id="theme-mode"
          aria-label="Theme mode"
          value={themeMode}
          className="w-[7.5rem] py-1.5 text-xs"
          onChange={(event) => onThemeModeChange(event.target.value as ThemeMode)}
        >
          <option value="dark">Dark</option>
          <option value="light">Light</option>
          <option value="system">System</option>
        </BrianSelect>
      </div>
      <div className="flex items-center gap-2">
        <BrianLabel htmlFor="theme-accent" className="text-xs text-[var(--text-secondary)]">
          Accent
        </BrianLabel>
        <BrianSelect
          id="theme-accent"
          aria-label="Accent palette"
          value={accent}
          className="w-[7.5rem] py-1.5 text-xs"
          onChange={(event) => onAccentChange(event.target.value as AccentPalette)}
        >
          {ACCENT_PALETTES.map((palette) => (
            <option key={palette} value={palette}>
              {ACCENT_LABELS[palette]}
            </option>
          ))}
        </BrianSelect>
      </div>
      <div className="flex items-center gap-2">{authSlot}</div>
    </div>
  );
}

export function AuthActions({
  signedIn,
  email,
  workspaceId,
  onSignIn,
  onSignOut,
}: {
  signedIn: boolean;
  email: string | null;
  workspaceId: string | null;
  onSignIn: () => void;
  onSignOut: () => void;
}) {
  if (signedIn) {
    return (
      <>
        <span
          className="max-w-[180px] truncate text-xs text-[var(--text-secondary)]"
          title={workspaceId ?? undefined}
        >
          {email ?? "Signed in"}
        </span>
        <BrianButton variant="secondary" className="py-1 text-xs" onClick={onSignOut}>
          Sign out
        </BrianButton>
      </>
    );
  }

  return (
    <BrianButton className="py-1 text-xs" onClick={onSignIn}>
      Sign in
    </BrianButton>
  );
}
