import ky from "ky";
import { useCallback } from "react";

import oliLogo from "../resources/OLI.png";
import kiltLogo from "../resources/built-on-kilt.svg";
import { API_URL } from "../api/backend";

function Footer() {
  // Callbacks
  const handleResetClick = useCallback(() => {
    if (!confirm("STOPP! Wirklich zurücksetzen?")) {
      return;
    }
    (async () => {
      await ky.delete(API_URL + "/did");
      window.location.reload();
    })();
  }, []);

  return (
    <div>
      <img
        src={oliLogo}
        alt="OLI logo"
        width={116}
        height={76}
        className="oli"
      />
      <img
        src={kiltLogo}
        alt="Built on KILT"
        width={142}
        height={28}
        className="kilt"
      />
      <button type="reset" onClick={handleResetClick}>
        Zurücksetzen
      </button>
    </div>
  );
}

export default Footer;
