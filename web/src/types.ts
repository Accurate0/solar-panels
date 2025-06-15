export interface GenerationHistory {
  wh: number;
  cummalativeKwh: number;
  at: string;
}

export interface SolarCurrentResponse {
  currentProductionWh: number;
  monthProductionKwh: number;
  todayProductionKwh: number;
  allTimeProductionKwh: number;
}

export interface SolarHistoryResponse {
  today: GenerationHistory[];
  yesterday: GenerationHistory[];
}
