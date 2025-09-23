import { createSignal, Show, createEffect, For, createMemo } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { SwipeableViews } from "~/components/Mobile/SwipeableViews";
import { SafeArea } from "~/components/Mobile/SafeArea";
import { authState, startGitHubAuth, startClaudeAuth } from "~/stores/authStore";
import { Toast } from "~/components/Common/Toast";
import { CheeseCelebration } from "~/components/Onboarding/CheeseCelebration";
import { ClaudeAuthSuccess } from "~/components/Onboarding/ClaudeAuthSuccess";
import "~/styles/onboarding.css";

type OnboardingStep = "welcome" | "github" | "claude" | "cheese";

const STEP_SEQUENCE: OnboardingStep[] = ["welcome", "github", "claude", "cheese"];

const STEP_LABELS: Record<OnboardingStep, string> = {
  welcome: "Intro",
  github: "GitHub",
  claude: "Claude",
  cheese: "Celebrate"
};

const FEATURE_CARDS = [
  {
    icon: "🤖",
    title: "AI Pairing",
    copy: "Draft prompts and review diffs with Claude Code right from your phone."
  },
  {
    icon: "📱",
    title: "Touch Ready",
    copy: "Large tap targets, swipe gestures, and safe areas tuned for mobile."
  },
  {
    icon: "🔒",
    title: "Secure Auth",
    copy: "Device flow keeps your GitHub & Claude credentials safe and revocable."
  },
  {
    icon: "⚡",
    title: "Ship Fast",
    copy: "Preview diffs, merge branches, and push commits in seconds."
  }
] as const;

const GITHUB_POINTS = [
  "Authorize via GitHub's device flow—no passwords or copy/paste required.",
  "We open a secure GitHub tab so you can paste the code shown below.",
  "Once connected, RAT syncs your repos and keeps auth tokens encrypted."
] as const;

const CLAUDE_POINTS = [
  "Boost code reviews with Claude Code completing edits alongside you.",
  "You can revoke access any time from your Anthropic dashboard.",
  "We only request scopes needed for real-time diffing and suggestions."
] as const;

export default function Onboarding() {
  const [currentStep, setCurrentStep] = createSignal<OnboardingStep>("welcome");
  const [error, setError] = createSignal<string | null>(null);
  const [showCelebration, setShowCelebration] = createSignal(false);
  const [showClaudeSuccess, setShowClaudeSuccess] = createSignal(false);
  const [hasCelebrated, setHasCelebrated] = createSignal(false);
  const navigate = useNavigate();

  const stepIndex = createMemo(() => {
    const index = STEP_SEQUENCE.indexOf(currentStep());
    return index === -1 ? 0 : index;
  });

  const progress = createMemo(() => {
    const denominator = STEP_SEQUENCE.length - 1;
    if (denominator <= 0) return 0;
    return Math.min(100, Math.max(0, (stepIndex() / denominator) * 100));
  });

  const progressWidth = createMemo(() => `${progress().toFixed(1)}%`);
  const currentLabel = createMemo(() => STEP_LABELS[currentStep()]);

  const goToStepByIndex = (index: number) => {
    const next = STEP_SEQUENCE[index];
    if (!next || showCelebration()) return;
    setCurrentStep(next);
  };

  const handleNext = (next: OnboardingStep) => {
    if (showCelebration()) return;
    setCurrentStep(next);
  };

  const handleComplete = () => {
    console.log("handleComplete called - navigating to dashboard");
    setShowCelebration(false);
    navigate("/dashboard");
  };

  createEffect(() => {
    if (authState.user?.githubConnected && currentStep() === "github") {
      setCurrentStep("claude");
    }
  });

  createEffect(() => {
    if (
      authState.user?.claudeConnected &&
      currentStep() === "claude" &&
      !showClaudeSuccess()
    ) {
      setShowClaudeSuccess(true);
    }
  });

  createEffect(() => {
    if (
      !hasCelebrated() &&
      authState.user?.githubConnected &&
      authState.user?.claudeConnected
    ) {
      setHasCelebrated(true);
      setCurrentStep("cheese");
      // Don't automatically show celebration, let user choose
    }
  });

  return (
    <SafeArea all>
      <div class="onboarding-shell">
        <div class="onboarding-aurora" aria-hidden="true"></div>
        <div class="onboarding-grid" aria-hidden="true"></div>
        <div class="relative flex min-h-screen flex-col">
          <header class="px-6 pt-10 pb-6 sm:pt-12">
            <div class="mx-auto w-full max-w-md space-y-4">
              <div class="flex items-center justify-between text-[11px] font-semibold uppercase tracking-[0.35em] text-white/50">
                <span>RAT Mobile IDE</span>
                <span>{currentLabel()}</span>
              </div>
              <div class="h-2 w-full overflow-hidden rounded-full bg-white/10">
                <div
                  class="h-full rounded-full bg-gradient-to-r from-sky-400 via-indigo-400 to-purple-500 transition-all duration-300 ease-out"
                  style={{ width: progressWidth() }}
                />
              </div>
              <div class="flex items-center justify-between">
                <For each={STEP_SEQUENCE}>
                  {(step, index) => (
                    <button
                      type="button"
                      class={`step-marker ${index() <= stepIndex() ? "active" : ""} ${index() === stepIndex() ? "current" : ""}`.trim()}
                      onClick={() => goToStepByIndex(index())}
                    >
                      {STEP_LABELS[step]}
                    </button>
                  )}
                </For>
              </div>
            </div>
          </header>

          <main class="flex-1">
            <div class="mx-auto h-full w-full max-w-md px-6 pb-24 sm:pb-16">
              <SwipeableViews
                index={stepIndex()}
                onIndexChange={goToStepByIndex}
                threshold={70}
                animateTransitions
              >
                <section class="flex h-full flex-col justify-between gap-10 py-2">
                  <div class="space-y-8">
                    <div class="onboarding-card p-6 sm:p-8 space-y-6">
                      <div class="inline-flex items-center gap-3 rounded-full bg-white/10 px-4 py-2 text-xs font-semibold uppercase tracking-[0.3em] text-white/70">
                        <span>Fresh</span>
                        <span>Just shipped</span>
                      </div>
                      <div class="space-y-4">
                        <h1 class="text-4xl font-bold leading-tight text-white sm:text-5xl">
                          Build & ship from anywhere.
                        </h1>
                        <p class="text-base text-white/70 sm:text-lg">
                          RAT is your pocket IDE. Pair with Claude, review diffs, and push commits without cracking open a laptop.
                        </p>
                      </div>
                    </div>
                    <div class="grid grid-cols-2 gap-3 sm:gap-4">
                      <For each={FEATURE_CARDS}>
                        {(feature) => (
                          <div class="onboarding-card p-4 text-left">
                            <div class="text-2xl">{feature.icon}</div>
                            <p class="mt-2 text-sm font-semibold text-white">
                              {feature.title}
                            </p>
                            <p class="mt-1 text-xs leading-relaxed text-white/60">
                              {feature.copy}
                            </p>
                          </div>
                        )}
                      </For>
                    </div>
                  </div>

                  <div class="space-y-3">
                    <button
                      class="primary-button w-full"
                      onClick={() => handleNext("github")}
                    >
                      Get started
                    </button>
                    <button
                      class="secondary-button w-full"
                      onClick={() => navigate("/dashboard")}
                    >
                      I already have an account
                    </button>
                  </div>
                </section>

                <section class="flex h-full flex-col justify-between gap-10 py-2">
                  <div class="onboarding-card p-6 sm:p-8 space-y-6">
                    <div class="flex items-center gap-4">
                      <div class="flex h-14 w-14 items-center justify-center rounded-2xl bg-white/10">
                        <svg class="h-7 w-7 text-white" viewBox="0 0 24 24" fill="currentColor">
                          <path d="M12 0C5.373 0 0 5.373 0 12c0 5.302 3.438 9.8 8.207 11.387.6.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.302 1.23a11.52 11.52 0 012.718-.316 11.52 11.52 0 012.718.316c2.293-1.552 3.301-1.23 3.301-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .318.192.693.801.575C20.565 21.8 24 17.302 24 12c0-6.627-5.373-12-12-12z" />
                        </svg>
                      </div>
                      <div class="space-y-1">
                        <h2 class="text-2xl font-semibold text-white">Connect GitHub</h2>
                        <p class="text-sm text-white/70">Stay signed in with GitHub's secure device flow.</p>
                      </div>
                    </div>
                    <p class="text-sm leading-relaxed text-white/65">
                      Approve RAT on GitHub in a new tab, then return here to finish linking your account.
                    </p>
                    <Show when={authState.githubUserCode}>
                      <div class="device-code-card">
                        <small class="text-xs uppercase tracking-[0.35em] text-emerald-200/80">Enter this code</small>
                        <code>{authState.githubUserCode}</code>
                        <Show when={authState.githubVerificationUri}>
                          <span class="text-xs text-emerald-100/70 break-all">
                            {authState.githubVerificationUri}
                          </span>
                        </Show>
                      </div>
                    </Show>
                    <ul class="space-y-2 text-sm text-white/70">
                      <For each={GITHUB_POINTS}>
                        {(point) => (
                          <li class="flex items-start gap-3">
                            <span class="mt-1 inline-flex h-2.5 w-2.5 flex-shrink-0 rounded-full bg-emerald-400/70"></span>
                            <span>{point}</span>
                          </li>
                        )}
                      </For>
                    </ul>
                    <div class="space-y-3 pt-2">
                      <button
                        class="primary-button w-full"
                        disabled={authState.isLoading}
                        onClick={async () => {
                          try {
                            setError(null);
                            const result = await startGitHubAuth();
                            if (result.user_code) {
                              setError(`Enter code: ${result.user_code}`);
                            }
                          } catch (err) {
                            console.error(err);
                            setError("Failed to connect GitHub. Please try again.");
                          }
                        }}
                      >
                        <Show
                          when={!authState.isLoading}
                          fallback={
                            <span class="flex items-center justify-center gap-3">
                              <svg class="h-5 w-5 animate-spin text-white" viewBox="0 0 24 24" fill="none">
                                <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2" opacity="0.2" />
                                <path d="M4 12a8 8 0 018-8" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                              </svg>
                              Connecting...
                            </span>
                          }
                        >
                          <span class="flex items-center justify-center gap-3">
                            <svg class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
                              <path d="M12 0C5.373 0 0 5.373 0 12c0 5.302 3.438 9.8 8.207 11.387.6.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.302 1.23a11.52 11.52 0 012.718-.316 11.52 11.52 0 012.718.316c2.293-1.552 3.301-1.23 3.301-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .318.192.693.801.575C20.565 21.8 24 17.302 24 12c0-6.627-5.373-12-12-12z" />
                            </svg>
                            Continue with GitHub
                          </span>
                        </Show>
                      </button>
                      <button
                        class="tertiary-button w-full"
                        type="button"
                        onClick={() => navigate("/dashboard")}
                      >
                        Skip for now
                      </button>
                    </div>
                  </div>
                </section>

                <section class="flex h-full flex-col justify-between gap-10 py-2">
                  <div class="onboarding-card p-6 sm:p-8 space-y-6">
                    <div class="flex items-center gap-4">
                      <div class="flex h-14 w-14 items-center justify-center rounded-2xl bg-orange-500/20 text-3xl">
                        🤖
                      </div>
                      <div class="space-y-1">
                        <h2 class="text-2xl font-semibold text-white">Connect Claude Code</h2>
                        <p class="text-sm text-white/70">Unlock AI pair programming that understands your repo.</p>
                      </div>
                    </div>
                    <p class="text-sm leading-relaxed text-white/65">
                      We'll launch Anthropic's device flow so Claude can suggest diffs, explain changes, and spot regressions in context.
                    </p>
                    <ul class="space-y-2 text-sm text-white/70">
                      <For each={CLAUDE_POINTS}>
                        {(point) => (
                          <li class="flex items-start gap-3">
                            <span class="mt-1 inline-flex h-2.5 w-2.5 flex-shrink-0 rounded-full bg-orange-300/80"></span>
                            <span>{point}</span>
                          </li>
                        )}
                      </For>
                    </ul>
                    <div class="space-y-3 pt-2">
                      <button
                        class="primary-button w-full"
                        disabled={authState.isLoading}
                        onClick={async () => {
                          try {
                            setError(null);
                            await startClaudeAuth();
                          } catch (err) {
                            console.error(err);
                            setError("Failed to connect Claude Code. Please try again.");
                          }
                        }}
                      >
                        <Show
                          when={!authState.isLoading}
                          fallback={
                            <span class="flex items-center justify-center gap-3">
                              <svg class="h-5 w-5 animate-spin text-white" viewBox="0 0 24 24" fill="none">
                                <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2" opacity="0.2" />
                                <path d="M4 12a8 8 0 018-8" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                              </svg>
                              Connecting...
                            </span>
                          }
                        >
                          Connect Claude Code
                        </Show>
                      </button>
                      <button
                        class="tertiary-button w-full"
                        type="button"
                        onClick={() => navigate("/dashboard")}
                      >
                        Skip for now
                      </button>
                    </div>
                  </div>
                </section>

                <section class="flex h-full flex-col justify-between gap-10 py-2">
                  <div class="onboarding-card p-6 sm:p-8 text-center space-y-6">
                    <div class="mx-auto flex h-16 w-16 items-center justify-center rounded-2xl bg-yellow-400/10 text-4xl">
                      🧀
                    </div>
                    <div class="space-y-2">
                      <h2 class="text-3xl font-semibold text-white">Say Cheese</h2>
                      <p class="text-sm text-white/70">
                        Launch the cheese confetti finale and tap anywhere to continue to your workspace.
                      </p>
                    </div>
                    <div class="space-y-3">
                      <button
                        class="primary-button w-full"
                        onClick={() => setShowCelebration(true)}
                        type="button"
                      >
                        Celebrate & continue
                      </button>
                      <button
                        class="ghost-button w-full"
                        type="button"
                        onClick={handleComplete}
                      >
                        Skip celebration
                      </button>
                    </div>
                    <p class="text-xs text-white/55">
                      Pro tip: you can revisit onboarding anytime from Settings → Experience.
                    </p>
                  </div>
                </section>
              </SwipeableViews>
            </div>
          </main>

          <Show when={showClaudeSuccess()}>
            <ClaudeAuthSuccess onComplete={() => setShowClaudeSuccess(false)} />
          </Show>

          <Show when={showCelebration()}>
            <CheeseCelebration onComplete={handleComplete} />
          </Show>

          <Show when={error()}>
            <Toast message={error()!} type="error" onClose={() => setError(null)} />
          </Show>
        </div>
      </div>
    </SafeArea>
  );
}
