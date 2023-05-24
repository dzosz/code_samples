#include "lockable_item.h"

lockable_item::lockable_item() : m_item_id(++sm_item_id_generator)
{};

item_id_type lockable_item::id() const noexcept { return m_item_id;}

tsv_type lockable_item::last_tsv() const noexcept { return m_last_tsv;
}


lockable_item::atomic_item_id   lockable_item::sm_item_id_generator{0};
