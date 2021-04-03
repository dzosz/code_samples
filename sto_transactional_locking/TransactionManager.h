#ifndef TRANSANCTIONMANAGER_H
#define TRANSANCTIONMANAGER_H

#include <atomic>
#include <condition_variable>
#include <vector>
#include <mutex>


#include "lockable_item.h"

class TransactionManager
{
public:
    // only by owning thread
    void begin();
    void commit();
    void rollback();

    TransactionManager(int log_level);

    bool  acquire(lockable_item& item);

    tx_id_type id() const;
    tsv_type tsv() const;

private:
    using item_ptr_list = std::vector<lockable_item*>;
    using mutex = std::mutex;
    using tx_lock = std::unique_lock<std::mutex>;
    using cond_var = std::condition_variable;
    using atomic_tsv = std::atomic<tsv_type>;
    using atomic_tx_id = std::atomic<tx_id_type>;

    tx_id_type m_tx_id;
    tsv_type m_tx_tsv = 0;
    item_ptr_list m_item_ptrs{};
    mutex m_mutex{};
    cond_var m_cond{};
    //FILE* m_fp;
    int m_log_Level;

    static atomic_tsv sm_tsv_generator;
    static atomic_tx_id sm_tx_id_generator;
};

#endif // TRANSANCTIONMANAGER_H
