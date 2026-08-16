import { useState } from "react";
import { api } from "../lib/api";
import type { Tag } from "../lib/types";
import { Button, ErrorBanner, Input, Modal, TagChip } from "./ui";

const DEFAULT_COLORS = ["#f5a623", "#4c8dff", "#f0555a", "#34c77b", "#a78bfa", "#f472b6"];

export default function ManageTagsModal({
  projectId,
  tags,
  onClose,
  onChanged,
}: {
  projectId: string;
  tags: Tag[];
  onClose: () => void;
  onChanged: () => void;
}) {
  const [name, setName] = useState("");
  const [color, setColor] = useState(DEFAULT_COLORS[0]);
  const [error, setError] = useState<string | null>(null);

  const create = async () => {
    if (!name.trim()) return;
    setError(null);
    try {
      await api.projects.createTag(projectId, name.trim(), color);
      setName("");
      onChanged();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to create tag");
    }
  };

  return (
    <Modal title="Project tags" onClose={onClose}>
      <div className="space-y-4">
        <div className="flex flex-wrap gap-1.5">
          {tags.map((tag) => (
            <TagChip
              key={tag.id}
              name={tag.name}
              color={tag.color}
              onRemove={async () => {
                await api.projects.deleteTag(projectId, tag.id);
                onChanged();
              }}
            />
          ))}
          {tags.length === 0 && <span className="text-xs text-text-faint">No tags yet</span>}
        </div>

        <div className="border-t border-border pt-3">
          <div className="flex items-center gap-2">
            <div className="flex gap-1">
              {DEFAULT_COLORS.map((c) => (
                <button
                  key={c}
                  onClick={() => setColor(c)}
                  className="h-6 w-6 rounded-full ring-1 ring-black/20"
                  style={{ backgroundColor: c, outline: color === c ? `2px solid ${c}` : undefined, outlineOffset: 2 }}
                  aria-label={`Color ${c}`}
                />
              ))}
            </div>
            <Input
              placeholder="Tag name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && create()}
            />
            <Button onClick={create} disabled={!name.trim()}>
              Add
            </Button>
          </div>
          {error && (
            <div className="mt-2">
              <ErrorBanner message={error} />
            </div>
          )}
        </div>
      </div>
    </Modal>
  );
}
