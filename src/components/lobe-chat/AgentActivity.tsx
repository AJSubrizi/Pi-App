/**
 * Mid-stream tool activity — plain one-line text (Codex-style).
 *
 * Rules:
 * - Only the latest **running** tool is shown
 * - Multiple tools replace the same line (no stack)
 * - Line sits in the stream (after current reply / at live edge)
 * - Hidden when no running tool (content can resume without chrome)
 * - Historical tool_step rows are not rendered in the transcript
 */

import { useMemo } from "react";
import type { Locale } from "@/i18n";
import { createT } from "@/i18n";
import type { ChatMessage } from "@/lib/session";
import { toolStepDisplayTitle } from "@/lib/session";
import { IconStop } from "@/components/icons";
// locale kept on LiveToolText for API compatibility with ConversationThread

export {
  isToolStepMessage,
  pickLatestTurnTool,
  pickRunningTurnTool,
  toolStepDisplayTitle,
} from "@/lib/session";

/**
 * Mid-stream tool status — plain call text only (no "tool" chrome).
 * Hidden when there is no meaningful title yet.
 */
export function LiveToolText({
  message,
  locale: _locale,
}: {
  message: ChatMessage;
  locale: Locale;
}) {
  const title = toolStepDisplayTitle(message);
  if (!title) return null;

  return (
    <div
      className="lobe-chat-tool-text"
      role="status"
      aria-live="polite"
      data-tool-id={message.toolCallId}
      title={message.toolDetail || message.toolPath || title}
    >
      {title}
    </div>
  );
}

export function TurnCancelledRow({
  message,
  locale,
}: {
  message: ChatMessage;
  locale: Locale;
}) {
  const tr = useMemo(() => createT(locale), [locale]);
  const reason = message.toolStatus || "";
  const label =
    reason === "user_stop"
      ? tr("activity.cancelledByUser")
      : reason === "agent_exit"
        ? tr("activity.cancelledAgentExit")
        : tr("activity.cancelled");
  return (
    <div className="lobe-chat-live-tool lobe-chat-live-tool--cancel" role="status">
      <span className="lobe-chat-live-tool__mark" aria-hidden>
        <IconStop size={13} />
      </span>
      <span className="lobe-chat-live-tool__title">{label}</span>
    </div>
  );
}
