import "./StatusCurrentDisplay.css";
import { useMemo } from "react";
import { formatTimeDiff } from "../../lib/time";
import ClientVersionLabel from "../../components/ClientVersionLabel";

type Period = {
  id: string;
  startTime: number;
  person: {
    id: string;
    firstName: string;
    lastName: string;
  };
};

function getSignInColor(startTime: number): string {
  const elapsedSeconds = Date.now() / 1000 - startTime;
  if (elapsedSeconds <= 60 * 60 * 6) return "status-good";
  if (elapsedSeconds <= 60 * 60 * 8) return "status-warning1";
  if (elapsedSeconds <= 60 * 60 * 10) return "status-warning2";
  if (elapsedSeconds <= 60 * 60 * 12) return "status-warning3";
  return "status-problem";
}

type Props = {
  periods: Period[];
};

export default function StatusCurrentDisplay({ periods }: Props) {
  const sortedPeriods = useMemo(
    () => [...periods].sort((a, b) => a.startTime - b.startTime),
    [periods],
  );

  return (
    <div id="status-current">
      <ul className="member-list">
        {sortedPeriods.map((period) => (
          <li key={period.id} className="member-name">
            <span className="member-label">
              {period.person.firstName} {period.person.lastName}
            </span>
            <span className={getSignInColor(period.startTime)}>
              {formatTimeDiff(new Date(period.startTime * 1000), new Date())}
            </span>
          </li>
        ))}
      </ul>
      <div className="member-count">
        {periods.length} member{periods.length !== 1 ? "s" : ""} signed in
      </div>
      <div className="status-version">
        <ClientVersionLabel noLink />
      </div>
    </div>
  );
}
