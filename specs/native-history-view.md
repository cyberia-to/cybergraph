# Complete native signal history view

NativeNode supplies a bounded history view from its existing canonical records.
Each entry retains its original receipt and operation bytes, and exposes the
complete committed Signals. Event batches use their exact encoded Signals;
Link/Pay operations use the corresponding committed block Signal. Intent and
host-only LocalCredit remain in original operation bytes and do not become peer
Signals. Reading this view neither applies a command again nor changes history.

This view is suitable for a graph observation projection. It is not authorization
to execute a copied host credit, nor a consensus/finality certificate. Consumers
retain source, original operation, receipt and full signal proof/network bytes.
They must not rebuild signals by taking only links from legacy tape or assign
fresh author/step/previous values to fill gaps.
