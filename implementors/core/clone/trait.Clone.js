(function() {var implementors = {};
implementors["aero"] = [{"text":"impl Clone for IDTFlags","synthetic":false,"types":[]},{"text":"impl Clone for IDTEntry","synthetic":false,"types":[]},{"text":"impl Clone for VirtualAddress","synthetic":false,"types":[]},{"text":"impl Clone for ScreenChar","synthetic":false,"types":[]},{"text":"impl Clone for Color","synthetic":false,"types":[]},{"text":"impl Clone for ColorCode","synthetic":false,"types":[]}];
implementors["bootloader"] = [{"text":"impl Clone for MemoryRegion","synthetic":false,"types":[]},{"text":"impl Clone for FrameRange","synthetic":false,"types":[]},{"text":"impl Clone for MemoryRegionType","synthetic":false,"types":[]},{"text":"impl Clone for TlsTemplate","synthetic":false,"types":[]}];
implementors["volatile"] = [{"text":"impl&lt;T:&nbsp;Copy&gt; Clone for Volatile&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Clone + Copy&gt; Clone for ReadOnly&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Clone + Copy&gt; Clone for WriteOnly&lt;T&gt;","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()