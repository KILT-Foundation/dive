import { Mode } from "../App";

export function setMode(mode: Mode) {
  console.log("Setting mode to", mode);
  localStorage.setItem("mode", mode);
}

export function getMode(): Mode {
  return (localStorage.getItem("mode") as Mode) || Mode.presentation;
}
