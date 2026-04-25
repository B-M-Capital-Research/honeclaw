// Mirror of backend `validate_password_strength` in routes/public.rs.
// Length 8..=128, must contain at least one digit AND one ASCII letter.

export const PASSWORD_MIN_LENGTH = 8;
export const PASSWORD_MAX_LENGTH = 128;

export type PasswordCheck = {
  ok: boolean;
  reason?: string;
  rules: {
    lengthOk: boolean;
    hasDigit: boolean;
    hasLetter: boolean;
  };
};

export function checkPasswordStrength(plain: string): PasswordCheck {
  const lengthOk = plain.length >= PASSWORD_MIN_LENGTH && plain.length <= PASSWORD_MAX_LENGTH;
  const hasDigit = /[0-9]/.test(plain);
  const hasLetter = /[A-Za-z]/.test(plain);
  const rules = { lengthOk, hasDigit, hasLetter };

  if (!lengthOk) {
    return {
      ok: false,
      reason: `密码长度需 ${PASSWORD_MIN_LENGTH}–${PASSWORD_MAX_LENGTH} 位`,
      rules,
    };
  }
  if (!hasDigit) {
    return { ok: false, reason: "密码至少包含一位数字", rules };
  }
  if (!hasLetter) {
    return { ok: false, reason: "密码至少包含一位字母", rules };
  }
  return { ok: true, rules };
}
