import { useCallback } from "react";

import { api } from "../api/backend";

export function AdminComponent() {
  // Callbacks
  const handleResetClick = useCallback(() => {
    if (!confirm("STOPP! Wirklich zurücksetzen?")) {
      return;
    }
    (async () => {
      await api.delete("did");
      window.location.reload();
    })();
  }, []);

  return (
    <button onClick={handleResetClick}>Zurücksetzen</button>
  );
}
