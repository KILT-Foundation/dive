import { useCallback } from "react";
import ky from "ky";

import { API_URL } from "../api/backend";

export function AdminComponent() {
  // Callbacks
  const handleResetClick = useCallback(() => {
    if (!confirm("STOPP! Wirklich zurücksetzen?")) {
      return;
    }
    (async () => {
      await ky.delete(API_URL + "/api/v1/did");
      window.location.reload();
    })();
  }, []);

  return (
    <button onClick={handleResetClick}>Zurücksetzen</button>
  );
}
