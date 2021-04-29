initSidebarItems({"constant":[["APIC_SPURIOUS_VECTOR",""],["XAPIC_EOI","EOI register. Write-only."],["XAPIC_ICR0","Interrupt Command Register (ICR). Read/write."],["XAPIC_ICR1","Interrupt Command Register (ICR). Read/write."],["XAPIC_SVR","Spurious Interrupt Vector Register (SVR). Read/write."],["XAPIC_TPR","Task Priority Register (TPR). Read/write. Bits 31:8 are reserved."]],"enum":[["ApicType",""]],"fn":[["ap_ready","Returns true if the AP is ready."],["get_bsp_id","Get the local BSP’s id."],["get_cpu_count","Return the number of active CPUs"],["get_local_apic","Get a mutable reference to the local apic."],["init","Initialize the local apic."]],"static":[["AP_READY",""],["BSP_APIC_ID",""],["CPU_COUNT","The count of all the active CPUs."],["LOCAL_APIC",""]],"struct":[["LocalApic",""]]});