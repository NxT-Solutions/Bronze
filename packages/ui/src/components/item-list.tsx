import { Button } from "@/components/button";

export type QueueItem = {
  id: string;
  body: string;
};

export const QUEUE_ACTION_KEYS = {
  moveUp: "queue.item.moveUp",
  moveDown: "queue.item.moveDown",
  complete: "queue.item.complete",
  skip: "queue.item.skip",
  trash: "queue.item.trash",
  edit: "queue.item.edit",
} as const;

export function ItemList({
  items,
  labels,
  onAction,
}: {
  items: QueueItem[];
  labels: Record<keyof typeof QUEUE_ACTION_KEYS, string>;
  onAction: (id: string, action: keyof typeof QUEUE_ACTION_KEYS) => void;
}) {
  return (
    <ul data-slot="item-list">
      {items.map((item) => (
        <li key={item.id}>
          <article>
            <p>{item.body}</p>
            <menu>
              {(
                [
                  "moveUp",
                  "moveDown",
                  "complete",
                  "skip",
                  "trash",
                  "edit",
                ] as const
              ).map((action) => (
                <Button
                  key={action}
                  type="button"
                  onPress={() => onAction(item.id, action)}
                >
                  {labels[action]}
                </Button>
              ))}
            </menu>
          </article>
        </li>
      ))}
    </ul>
  );
}
