import oliLogo from "../resources/OLI.png";
import kiltLogo from "../resources/built-on-kilt.svg";

function Footer() {
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
    </div>
  );
}

export default Footer;
