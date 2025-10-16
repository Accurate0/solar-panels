import { useLoaderData } from "react-router";
import "./App.css";
import * as React from "react";
import { Line, LineChart, XAxis, YAxis, ReferenceLine } from "recharts";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  type ChartConfig,
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart";
import type { loader } from "./loader";

export const description = "An interactive line chart";

const uvLevelColour = "#43A047";
const chartConfig = {
  views: {
    label: "Solar generation",
  },
  today: {
    label: "Today",
    color: "var(--chart-1)",
  },
  yesterday: {
    label: "Yesterday",
    color: "#1976D2",
  },
} satisfies ChartConfig;

export function ChartLineInteractive() {
  const { today, yesterday, current } = useLoaderData<typeof loader>();
  const [activeChart, setActiveChart] =
    React.useState<keyof typeof chartConfig>("today");

  const now = new Date();
  const endOfDay = new Date(
    now.getUTCFullYear(),
    now.getUTCMonth(),
    now.getUTCDate(),
    23,
    55,
  );

  const total = React.useMemo(
    () => ({
      today: current.todayProductionKwh,
      yesterday: current.yesterdayProductionKwh,
    }),
    [current.todayProductionKwh, current.yesterdayProductionKwh],
  );

  return (
    <div>
      <h1 className="scroll-m-20 text-left text-4xl font-bold tracking-tight text-balance">
        Solar panels
      </h1>
      <div className="grid grid-rows-2 gap-4 py-4">
        <div className="grid grid-cols-3 gap-4">
          <Card>
            <CardHeader>
              <CardDescription>Current</CardDescription>
              <CardTitle className="xs: text-sm md:text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
                {current.currentProductionWh} Wh
              </CardTitle>
            </CardHeader>
          </Card>

          <Card>
            <CardHeader>
              <CardDescription>Month</CardDescription>
              <CardTitle className="xs: text-sm md:text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
                {current.monthProductionKwh} kWh
              </CardTitle>
            </CardHeader>
          </Card>

          <Card>
            <CardHeader>
              <CardDescription>All Time</CardDescription>
              <CardTitle className="xs:text-sm md:text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
                {current.allTimeProductionKwh} kWh
              </CardTitle>
            </CardHeader>
          </Card>
        </div>

        <div className="grid grid-cols-3 gap-4">
          <Card>
            <CardHeader>
              <CardDescription>15m (avg)</CardDescription>
              <CardTitle className="xs:text-sm md:text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
                {current.statistics.averages.last15Mins.toFixed(0)} Wh
              </CardTitle>
            </CardHeader>
          </Card>

          <Card>
            <CardHeader>
              <CardDescription>1h (avg)</CardDescription>
              <CardTitle className="xs:text-sm md:text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
                {current.statistics.averages.last1Hour.toFixed(0)} Wh
              </CardTitle>
            </CardHeader>
          </Card>

          <Card>
            <CardHeader>
              <CardDescription>3h (avg)</CardDescription>
              <CardTitle className="xs:text-sm md:text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
                {current.statistics.averages.last3Hours.toFixed(0)} Wh
              </CardTitle>
            </CardHeader>
          </Card>
        </div>
      </div>
      <Card className="py-4 sm:py-0">
        <CardHeader className="flex flex-col items-stretch border-b !p-0 sm:flex-row">
          <div className="flex flex-1 flex-col justify-center gap-1 px-6 pb-3 sm:pb-0">
            <CardTitle>Solar Generation (24hr)</CardTitle>
          </div>
          <div className="flex">
            {["today", "yesterday"].map((key) => {
              const chart = key as keyof typeof chartConfig;
              return (
                <button
                  key={chart}
                  data-active={activeChart === chart}
                  className="data-[active=true]:bg-muted/50 flex flex-1 flex-col justify-center gap-1 border-t px-6 py-4 text-left even:border-l sm:border-t-0 sm:border-l sm:px-8 sm:py-4"
                  onClick={() => setActiveChart(chart)}
                >
                  <span className="text-muted-foreground text-xs">
                    {chartConfig[chart].label}
                  </span>
                  <span className="text-lg leading-none font-bold sm:text-xl">
                    {total[key as keyof typeof total].toLocaleString()} kWh
                  </span>
                </button>
              );
            })}
          </div>
        </CardHeader>
        <CardContent className="px-2 sm:p-6" style={{ paddingTop: 0 }}>
          <ChartContainer
            config={chartConfig}
            className="aspect-auto h-[250px] w-full"
          >
            <LineChart
              accessibilityLayer
              data={activeChart === "today" ? today : yesterday}
              margin={{
                left: 12,
                right: 12,
              }}
            >
              {[1000, 2000, 3000, 4000, 5000, 6000].map((y) => (
                <ReferenceLine key={y} y={y} stroke="#ccc" strokeWidth={0.5} />
              ))}
              <YAxis
                axisLine={false}
                tickLine={false}
                domain={[0, 6500]}
                ticks={[1000, 2000, 3000, 4000, 5000, 6000]}
                interval={"preserveStartEnd"}
              />
              <YAxis hide={true} yAxisId="uvLevelAxis" domain={[0, 13]} />
              <XAxis
                dataKey="timestamp"
                tickLine={false}
                interval={"preserveStart"}
                type="number"
                scale="time"
                domain={[
                  "dataMin",
                  activeChart === "today" ? endOfDay.getTime() : "auto",
                ]}
                axisLine={false}
                tickMargin={8}
                minTickGap={32}
                tickFormatter={(value) => {
                  const date = new Date(value);
                  return date.toLocaleTimeString("en-US", {
                    hour: "numeric",
                    minute: "numeric",
                  });
                }}
              />
              <ChartTooltip
                content={
                  <ChartTooltipContent
                    className="w-[150px]"
                    labelKey="timestamp"
                    labelFormatter={(value) => {
                      return new Date(value).toLocaleTimeString("en-US", {
                        hour: "numeric",
                        minute: "numeric",
                      });
                    }}
                  />
                }
              />

              <Line
                dataKey="uvLevel"
                name="UV Level"
                type="monotone"
                strokeDasharray="3 3"
                stroke={uvLevelColour}
                dot={false}
                yAxisId={"uvLevelAxis"}
              />
              <Line
                dataKey="wh"
                name="Wh"
                type="monotone"
                stroke={`var(--color-${activeChart})`}
                strokeWidth={2}
                dot={false}
              />
            </LineChart>
          </ChartContainer>
        </CardContent>
      </Card>
      <p className="text-muted-foreground text-xs text-left pt-3">
        UV observations courtesy of ARPANSA
      </p>
    </div>
  );
}

export const App = () => {
  return (
    <div>
      <ChartLineInteractive />
    </div>
  );
};
