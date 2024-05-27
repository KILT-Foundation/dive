import { useCallback } from "react";
import ky from "ky";

import { API_URL } from "../api/backend";
import { Mode } from "../App";

export function AdminComponent({
  mode,
  handleModeSwitch,
}: {
  mode: Mode;
  handleModeSwitch: () => void;
}) {
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
    <>
      <button style={{ margin: "10px" }} onClick={handleModeSwitch}>
        {mode}
      </button>
      <button onClick={handleResetClick}>Zurücksetzen</button>
    </>
  );
}
