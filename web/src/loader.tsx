import type { SolarCurrentResponse, SolarHistoryResponse } from "./types";

export const loader = async () => {
  const historyPromise = fetch(`${import.meta.env.VITE_API_BASE_URL}/history`)
    .then((r) => r.json())
    .then((json) => json as SolarHistoryResponse);

  const currentPromise = fetch(`${import.meta.env.VITE_API_BASE_URL}/current`)
    .then((r) => r.json())
    .then((json) => json as SolarCurrentResponse);

  const [historyResponse, currentResponse] = await Promise.all([
    historyPromise,
    currentPromise,
  ]);

  return { ...historyResponse, current: currentResponse };
};
