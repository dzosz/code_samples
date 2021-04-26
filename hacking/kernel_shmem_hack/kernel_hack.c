#include <linux/highmem.h>
#include <linux/module.h>
#include <linux/proc_fs.h>
#include <linux/seq_file.h>
#include <linux/sched/signal.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Emma N. Skye");
MODULE_DESCRIPTION("A glow hack implemented in kernel space.");


/*
When we load our module later we’ll provide two arguments: one for the patch address – this will be relative to the base address of client_client.so in /proc/pid/maps, the other argument will be how many bytes to replace with NOP instructions
 */

//static ulong offset=0x60;
const static char to_find[40] = "this string was not hacked";
const static char to_replace[40] = "HACKED";
#define MAX_PAGES 4


/*
static uint size;
module_param_named(patch_size, size, uint, 0644);
MODULE_PARM_DESC(patch_size, "Amount of bytes to replace with NOP instruction.");
*/

//module_param_named(patch_offset, offset, ulong, 0644);
//MODULE_PARM_DESC(patch_offset, "Target patching address relative to your executable/library.");

/*
There are a few different ways for userspace to communicate with the kernel. In the few minutes I spent searching, I found that making an entry in /proc was the easiest. Here’s everything we’ll need for that.

Our code will be triggered by reading from /proc/kglow – pretty lame, but it works.
 */

static int kglow_proc_show(struct seq_file* m, void* v);

static int kglow_proc_open(struct inode* i, struct file* f) {
  return single_open(f, kglow_proc_show, 0);
}

static const struct proc_ops kglow_proc_fops = {
  //.owner = THIS_MODULE,
  .proc_open = kglow_proc_open,
  .proc_read = seq_read,
  .proc_lseek = seq_lseek,
  .proc_release = single_release,
};

static int __init kglow_proc_init(void) {
  printk(KERN_INFO "KGLOW INIT to_find:'%s', to_replace='%s'", to_find, to_replace);
  proc_create("kglow", 0, 0, &kglow_proc_fops);
  return 0;
}

static void __exit kglow_proc_exit(void) {
  printk(KERN_INFO "KGLOW EXIT");
  remove_proc_entry("kglow", NULL);
}

module_init(kglow_proc_init);
module_exit(kglow_proc_exit);


/*
Fairly straight-forward so far. We use the for_each_process macro to enumerate the list of processes. In this case, processes are represented as a task_struct which contains a variable called comm – the base executable name.

Next on the list, finding the ‘base’ address of client_client.so. For this, I looked at the source of show_map_vma which is used for generating the /proc/pid/maps file. Everything we need is in the vm_area_struct, including the next one. Simply walk through the list, skipping those that aren’t backed by files.
*/

static int overwrite(struct task_struct* task, struct vm_area_struct* current_mmap);

static int kglow_proc_show(struct seq_file* f, void* v) {
    int i = 0;
  struct task_struct* task;

  const char* targetProcess = "target_process";
  for_each_process(task) {
      if (strcmp(task->comm, targetProcess) == 0) {
          struct vm_area_struct* current_mmap;
          printk(KERN_INFO "Got %s pid: %d\n", targetProcess, task->pid);
          printk(KERN_INFO "code: %px-%px\n", task->mm->start_code, task->mm->end_code);
          printk(KERN_INFO "data: %px-%px\n", task->mm->start_data, task->mm->end_data);
          printk(KERN_INFO "stack: %px...\n", task->mm->start_stack);
          printk(KERN_INFO "pages: %px...\n", task->mm->total_vm);

          if (!task->mm || !task->mm->mmap) {
              printk(KERN_INFO "Something wrong! Zero mm or mm->mmap\n");
              continue;
          }
	  mmap_write_lock(task->mm); // write mutex

          for (current_mmap = task->mm->mmap; current_mmap; current_mmap = current_mmap->vm_next) {
              int numPages = (current_mmap->vm_end - current_mmap->vm_start)/PAGE_SIZE;
              char filename[100];
              char* ptr=filename;
              if(current_mmap->vm_file){
                  ptr += snprintf(filename, sizeof(filename),
				  current_mmap->vm_file->f_path.dentry->d_iname);
              }
              else{ // implies an anonymous mapping i.e. not file backup'ed
                  ptr += snprintf(filename, sizeof(filename), "anon");
              }

              if (current_mmap->vm_file == NULL) // TODO need to understand why we don't need that
		      continue;

	      // condition you know exactly where to look for
              //if (strcmp(current_mmap->vm_file->f_path.dentry->d_iname, "client_client.so") != 0)
              //    continue;

              if (current_mmap->vm_flags == VM_NONE)
                  ptr += snprintf(ptr, sizeof(filename) - (ptr-filename)," NONE");
              if (current_mmap->vm_flags == VM_READ)
                  ptr += snprintf(ptr, sizeof(filename) - (ptr-filename)," READ");
              if (current_mmap->vm_flags & VM_WRITE)
                  ptr += snprintf(ptr, sizeof(filename) - (ptr-filename)," WRITE");
              if (current_mmap->vm_flags & VM_EXEC)
                  ptr += snprintf(ptr, sizeof(filename) - (ptr-filename)," EXEC");
              if (current_mmap->vm_flags & VM_SHARED)
                  ptr +=snprintf(ptr, sizeof(filename) - (ptr-filename)," SHARED");

              printk(KERN_INFO "%d) %s, %d pages at %px-%px\n",
			      i, filename, numPages, current_mmap->vm_start, current_mmap->vm_end);

	      overwrite(task, current_mmap);

              //break; // ?
              ++i;
          }
	  mmap_write_unlock(task->mm);// write mutex
      }
  }

  return 0;
}

/*
Firstly, lock the semaphore in the vm_area_struct for writing, this is a requirement for calling the next function, get_user_pages_remote. This function is used to pin the target page into memory – check out this blog post for a much better explanation on pages and general memory management than you’re ever going to find here.

Once we have the page, we pass it to kmap. This function creates a special mapping in kernel address space for our page, allowing us to manipulate it directly. It’s pretty simple after that, we use a modulo operation on the user-provided offset to get the distance from the page – for example 0xC6E367 would become 0x367 on a system with a page size of 0x1000.

Since we have write access from passing FOLL_FORCE in the gup_flags parameter, we can simply call memset, write our NOP instructions and then unmap the page.

// jpeg_UtlBuffer_dest is exported at BE7B20 so add 86847 to get to C6E367
All done with that now, it’s time to test this out! Make sure you’ve noted down the patch address from earlier, you’ll need to convert it to decimal (in this case, C6E367 = 13034343) when you pass the patch_offset parameter to the module loader.
*/

char* find_string_in_memory(const char* to_find, const char* start, const char* end) {
    int len = strlen(to_find);
    int i =0;
    for (i=0; start < end-len; start++) {
        int n = memcmp(start, to_find, len);

        // MATCH!!
        if (!n) {
            return (char*)start;
        }
    }
    return NULL;
}

static int writeIteration=0;

static int overwrite(struct task_struct* task, struct vm_area_struct* current_mmap)
{
    char* string_location=NULL;
    struct page* pages[MAX_PAGES];
    int locked = 1;
    int numPages=0;
    int currentPage=0;

    // FOLL PIN because we potentially change data, for safety?
    // FOOL_WRITE instead FOLL_FORCE because FOLL_FORCE caused kernel to crash on pagefault
    numPages = pin_user_pages_remote(task->mm, current_mmap->vm_start, MAX_PAGES, FOLL_WRITE | FOLL_PIN | FOLL_REMOTE, pages, NULL, &locked);
    if (numPages <0) {
	    //printk(KERN_ERR "pin_user_pages_locked returned %d\n", numPages);
	    return 1;
    }

    if (!locked) {
	    printk(KERN_ERR "ERR SOME ISUEE NOT LOCKED\n");
	    unpin_user_pages(pages, numPages);
	    return 1;
    }
    printk(KERN_INFO "found %d pages at %px-%px\n", numPages, current_mmap->vm_start, current_mmap->vm_end);


    for (currentPage=0; currentPage < numPages; ++currentPage) {
        struct page* pg = pages[currentPage];
        char* target = kmap(pg);
        printk(KERN_INFO "Page %d) %px-%px\n", currentPage, target, target+PAGE_SIZE);

        // overwrite 6 bytes with the NOP instruction
        //memset(target + (offset % PAGE_SIZE), 0x90, size);

        if(!string_location) {
            string_location = find_string_in_memory(to_find, target, target+PAGE_SIZE);
            if (string_location) {
                printk(KERN_INFO "XYZ FOUND at %px, offset=%x, it=%d\n", string_location, (string_location-target), writeIteration);
		// OVERWRITE PROCESS MEMORY!!
                snprintf(string_location, sizeof(to_find), "%d %s", writeIteration++, to_replace);
            }
	    kunmap(pg);

	    // careful, I think this needs to be called after kunmap
	    if(string_location) {
		    set_page_dirty(pg);
	    }
        }
	else
	    kunmap(pg);
    }

    unpin_user_pages(pages, numPages);
    return 0;
}
