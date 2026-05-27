import ScanController from "../components/ScanController";
import ScanTitleBar from "../components/ScanTitleBar";
import ClientVersionLabel from "../../components/ClientVersionLabel";
import { useState } from "react";

export default function Scan() {
  const [cancelSignOut, setCancelSignOut] = useState<(() => void) | null>(null);
  const [signingOutName, setSigningOutName] = useState<string | null>(null);

  return (
    <div id="scan">
      <ScanTitleBar
        onCancelSignOut={cancelSignOut ?? undefined}
        signingOutName={signingOutName ?? undefined}
      />
      <div id="content">
        <ScanController
          onCancelSignOutChange={(fn) => setCancelSignOut(fn ? () => fn : null)}
          onSigningOutNameChange={setSigningOutName}
        />
      </div>
      <div id="scan-version">
        <ClientVersionLabel noLink />
      </div>
    </div>
  );
}
