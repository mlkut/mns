const CRED_SEP = "::";

function parseCredentialPassword(password) {
  if (!password) return null;
  const parts = password.split(CRED_SEP);
  if (parts.length !== 3 || !parts[0] || !parts[1] || !parts[2]) return null;
  var keyType = parseInt(parts[1], 16);
  if (isNaN(keyType) || parts[2].length !== 64) return null;
  return { rskHex: parts[0], keyType: keyType, keyHex: parts[2] };
}

export async function storeCredential(username, rskHex, keyType, keyHex) {
  if (!("PasswordCredential" in window)) return false;
  try {
    var keyTypeHex = keyType.toString(16).padStart(2, "0");
    const cred = new window.PasswordCredential({
      id: username,
      password: rskHex + CRED_SEP + keyTypeHex + CRED_SEP + keyHex,
      name: "MNS Wallet — " + username,
    });
    await navigator.credentials.store(cred);
    return true;
  } catch {
    return false;
  }
}

export async function getCredential(mediation) {
  if (!navigator.credentials || !("PasswordCredential" in window)) return null;
  var cred;
  try {
    cred = await navigator.credentials.get({
      password: true,
      mediation: mediation || "optional",
    });
  } catch {
    return null;
  }
  if (!cred || !cred.password) return null;
  var keys = parseCredentialPassword(cred.password);
  if (!keys) return null;
  keys.id = cred.id || "wallet";
  return keys;
}
