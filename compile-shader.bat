if not exist "target/shaders/" mkdir "target/shaders/"
if not exist "target/shaders/main" mkdir "target/shaders/main"
if not exist "target/shaders/ui" mkdir "target/shaders/ui"

"C:\VulkanSDK\1.4.328.1\Bin\glslc.exe" shaders/main/shader.vert -o target/shaders/main/vert.spv
"C:\VulkanSDK\1.4.328.1\Bin\glslc.exe" shaders/main/shader.frag -o target/shaders/main/frag.spv

"C:\VulkanSDK\1.4.328.1\Bin\glslc.exe" shaders/ui/shader.vert -o target/shaders/ui/vert.spv
"C:\VulkanSDK\1.4.328.1\Bin\glslc.exe" shaders/ui/shader.frag -o target/shaders/ui/frag.spv
