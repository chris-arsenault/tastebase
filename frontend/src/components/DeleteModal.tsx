import type { TastingRecord } from "../types";

type DeleteModalProps = {
  target: TastingRecord;
  deleting: boolean;
  onConfirm: () => void;
  onClose: () => void;
};

export function DeleteModal({
  target,
  deleting,
  onConfirm,
  onClose,
}: Readonly<DeleteModalProps>) {
  return (
    <div className="modal-overlay">
      <div className="modal" role="dialog" aria-modal="true">
        <div className="modal-header">
          <h3>Delete tasting?</h3>
          <button
            type="button"
            className="modal-close"
            onClick={onClose}
            aria-label="Close"
          >
            {"\u00D7"}
          </button>
        </div>
        <div className="modal-body">
          <p>
            This will permanently remove{" "}
            <strong>{target.name || "this tasting"}</strong>.
          </p>
          <p className="modal-warning">This action cannot be undone.</p>
        </div>
        <div className="modal-actions">
          <button
            type="button"
            className="btn-ghost"
            onClick={onClose}
            disabled={deleting}
          >
            Cancel
          </button>
          <button
            type="button"
            className="btn-danger"
            onClick={onConfirm}
            disabled={deleting}
          >
            {deleting ? "Deleting..." : "Delete"}
          </button>
        </div>
      </div>
    </div>
  );
}
