export interface GenerationHistory {
  wh: number;
  atUtc: string;
  timestamp: number;
}

export interface SolarCurrentResponse {
  currentProductionWh: number;
  monthProductionKwh: number;
  todayProductionKwh: number;
  yesterdayProductionKwh: number;
  allTimeProductionKwh: number;
  statistics: {
    averages: {
      last15Mins: number;
      last1Hour: number;
      last3Hours: number;
    };
  };
}

export interface SolarHistoryResponse {
  today: GenerationHistory[];
  yesterday: GenerationHistory[];
}
