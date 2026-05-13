import type { UserInfo } from "./types";

export function filterUsers(users: UserInfo[], query: string, channel = "all") {
  const normalizedQuery = query.trim().toLowerCase();
  return users.filter((user) => {
    const searchableUserFields = [
      user.session_label,
      user.user_id,
      user.actor_user_id ?? "",
      user.channel_scope ?? "",
      user.channel,
      user.session_kind,
    ]
      .join(" ")
      .toLowerCase();
    const matchesQuery =
      !normalizedQuery || searchableUserFields.includes(normalizedQuery);
    const matchesChannel =
      channel === "all" || (user.channel || "direct") === channel;
    return matchesQuery && matchesChannel;
  });
}

export function hasUnread(
  userId: string,
  lastTime: string,
  lastRole: string,
  readAt: Record<string, string>,
  currentUserId?: string,
) {
  if (currentUserId === userId) return false;
  const readStamp = readAt[userId];
  if (!readStamp) return lastRole === "user";
  return new Date(lastTime).getTime() > new Date(readStamp).getTime();
}
