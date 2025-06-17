import { useLoaderData } from "react-router";
import "./App.css";
import * as React from "react";
import { CartesianGrid, Line, LineChart, XAxis } from "recharts";

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
    color: "var(--chart-2)",
  },
} satisfies ChartConfig;

export function ChartLineInteractive() {
  const { today, yesterday, current } = useLoaderData<typeof loader>();
  const [activeChart, setActiveChart] =
    React.useState<keyof typeof chartConfig>("today");

  const total = React.useMemo(
    () => ({
      today: current.todayProductionKwh,
      yesterday: current.yesterdayProductionKwh,
    }),
    [current.todayProductionKwh, current.yesterdayProductionKwh],
  );

  console.log(current);

  return (
    <div>
      <h1 className="scroll-m-20 text-left text-4xl font-bold tracking-tight text-balance">
        Solar panels
      </h1>
      <div className="flex md:flex-row flex-1 gap-4 py-4 w-full flex-col">
        <Card className="flex-grow-1">
          <CardHeader>
            <CardDescription>Current</CardDescription>
            <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
              {current.currentProductionWh} Wh
            </CardTitle>
          </CardHeader>
        </Card>

        <Card className="flex-grow-1">
          <CardHeader>
            <CardDescription>Last 15 minutes (avg)</CardDescription>
            <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
              {current.statistics.averages.last15Mins} Wh
            </CardTitle>
          </CardHeader>
        </Card>

        <Card className="flex-grow-1">
          <CardHeader>
            <CardDescription>Last 1 hour (avg)</CardDescription>
            <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
              {current.statistics.averages.last1Hour} Wh
            </CardTitle>
          </CardHeader>
        </Card>

        <Card className="flex-grow-1">
          <CardHeader>
            <CardDescription>Last 3 hours (avg)</CardDescription>
            <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
              {current.statistics.averages.last3Hours} Wh
            </CardTitle>
          </CardHeader>
        </Card>

        <Card className="flex-grow-1">
          <CardHeader>
            <CardDescription>Month</CardDescription>
            <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
              {current.monthProductionKwh} kWh
            </CardTitle>
          </CardHeader>
        </Card>
        <Card className="flex-grow-1">
          <CardHeader>
            <CardDescription>All Time</CardDescription>
            <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
              {current.allTimeProductionKwh} kWh
            </CardTitle>
          </CardHeader>
        </Card>
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
        <CardContent className="px-2 sm:p-6">
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
              <CartesianGrid vertical={false} />
              <XAxis
                dataKey="atUtc"
                tickLine={false}
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
                    nameKey="wh"
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
                dataKey="wh"
                type="monotone"
                stroke={`var(--color-${activeChart})`}
                strokeWidth={2}
                dot={false}
              />
            </LineChart>
          </ChartContainer>
        </CardContent>
      </Card>
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
