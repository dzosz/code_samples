#ifndef LOCKABLE_ITEM_H
#define LOCKABLE_ITEM_H

#include <atomic>


using tsv_type = uint64_t; // timestamp value
using tx_id_type = uint64_t; // transaction id
using item_id_type = uint64_t; // item id

class TransactionManager;

// to be used as a base class
class lockable_item
{

public:
    lockable_item();

        item_id_type id() const noexcept;
        tsv_type last_tsv() const noexcept;

private:
        friend class TransactionManager;

        using atomic_txm_pointer = std::atomic<TransactionManager*>;
        using atomic_item_id = std::atomic<item_id_type>;

        atomic_txm_pointer mp_owning_tx{0}; // pointer to transaction manager that owns this item_id_type
        tsv_type m_last_tsv{0}; // timestamp of last owner
        item_id_type m_item_id; // for debugging/tracking/logging

        static atomic_item_id  sm_item_id_generator;
};

#endif // LOCKABLE_ITEM_H
