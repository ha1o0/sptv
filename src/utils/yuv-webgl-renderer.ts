export class YUVRenderer {
    constructor(canvas) {
        this.gl = canvas.getContext("webgl");
        this.initGL();
    }
  
    initGL() {
        const gl = this.gl;
        gl.clearColor(0, 0, 0, 1);
        gl.clear(gl.COLOR_BUFFER_BIT);
  
        // WebGL 着色器代码
        const vertexShaderSource = `
            attribute vec4 a_position;
            attribute vec2 a_texCoord;
            varying vec2 v_texCoord;
            void main() {
                gl_Position = a_position;
                v_texCoord = a_texCoord;
            }
        `;
  
        const fragmentShaderSource = `
            precision mediump float;
            varying vec2 v_texCoord;
            uniform sampler2D u_textureY;
            uniform sampler2D u_textureU;
            uniform sampler2D u_textureV;
            void main() {
                float y = texture2D(u_textureY, v_texCoord).r;
                float u = texture2D(u_textureU, v_texCoord).r - 0.5;
                float v = texture2D(u_textureV, v_texCoord).r - 0.5;
                float r = y + 1.402 * v;
                float g = y - 0.344 * u - 0.714 * v;
                float b = y + 1.772 * u;
                gl_FragColor = vec4(r, g, b, 1.0);
            }
        `;
  
        this.program = this.createShaderProgram(gl, vertexShaderSource, fragmentShaderSource);
        gl.useProgram(this.program);
  
        // 创建全屏绘制所需的顶点数据（包含位置和纹理坐标）
        // 使用 TRIANGLE_STRIP 绘制一个全屏四边形
        const vertices = new Float32Array([
            //  x,    y,    u,   v
            -1.0, -1.0,  0.0, 1.0, // 左下角
             1.0, -1.0,  1.0, 1.0, // 右下角
            -1.0,  1.0,  0.0, 0.0, // 左上角
             1.0,  1.0,  1.0, 0.0  // 右上角
        ]);
        this.vertexBuffer = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, this.vertexBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);
  
        // 获取属性位置并设置指针
        const a_position = gl.getAttribLocation(this.program, "a_position");
        const a_texCoord = gl.getAttribLocation(this.program, "a_texCoord");
        // 每个顶点由4个 float 组成：前两个为位置，后两个为纹理坐标
        gl.vertexAttribPointer(a_position, 2, gl.FLOAT, false, 4 * 4, 0);
        gl.enableVertexAttribArray(a_position);
        gl.vertexAttribPointer(a_texCoord, 2, gl.FLOAT, false, 4 * 4, 2 * 4);
        gl.enableVertexAttribArray(a_texCoord);
  
        // 将纹理绑定到着色器采样器
        gl.uniform1i(gl.getUniformLocation(this.program, "u_textureY"), 0);
        gl.uniform1i(gl.getUniformLocation(this.program, "u_textureU"), 1);
        gl.uniform1i(gl.getUniformLocation(this.program, "u_textureV"), 2);
  
        // 初始化纹理句柄
        this.yTexture = null;
        this.uTexture = null;
        this.vTexture = null;
    }
  
    createTexture(gl, width, height) {
        const texture = gl.createTexture();
        // 记录纹理尺寸，便于后续比较
        texture.width = width;
        texture.height = height;
        gl.bindTexture(gl.TEXTURE_2D, texture);
        // 此处创建空纹理，后续使用 texSubImage2D 更新数据
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.LUMINANCE, width, height, 0, gl.LUMINANCE, gl.UNSIGNED_BYTE, null);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
        return texture;
    }
  
    createShaderProgram(gl, vertexSource, fragmentSource) {
        function compileShader(gl, source, type) {
            const shader = gl.createShader(type);
            gl.shaderSource(shader, source);
            gl.compileShader(shader);
            if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
                console.error("Shader compile error:", gl.getShaderInfoLog(shader));
            }
            return shader;
        }
        const vertexShader = compileShader(gl, vertexSource, gl.VERTEX_SHADER);
        const fragmentShader = compileShader(gl, fragmentSource, gl.FRAGMENT_SHADER);
        const program = gl.createProgram();
        gl.attachShader(program, vertexShader);
        gl.attachShader(program, fragmentShader);
        gl.linkProgram(program);
        if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
            console.error("Program link error:", gl.getProgramInfoLog(program));
        }
        return program;
    }
  
    updateTexture(texture, index, width, height, data) {
        const gl = this.gl;
        if (!data || data.length === 0) {
            console.error("No pixel data to update texture.");
            return;
        }
        gl.activeTexture(gl.TEXTURE0 + index);
        gl.bindTexture(gl.TEXTURE_2D, texture);
        gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, width, height, gl.LUMINANCE, gl.UNSIGNED_BYTE, data);
    }
  
    renderFrame(yData, uData, vData, width, height) {
        const gl = this.gl;
        // 验证数据有效性
        if (!yData || !uData || !vData || yData.length === 0 || uData.length === 0 || vData.length === 0) {
            console.error("Y, U, or V data is empty!");
            return;
        }
  
        // 如果纹理不存在或尺寸不匹配则重新创建纹理
        if (!this.yTexture || this.yTexture.width !== width || this.yTexture.height !== height) {
            this.yTexture = this.createTexture(gl, width, height);
            this.uTexture = this.createTexture(gl, width / 2, height / 2);
            this.vTexture = this.createTexture(gl, width / 2, height / 2);
        }
  
        // 更新纹理数据
        this.updateTexture(this.yTexture, 0, width, height, yData);
        this.updateTexture(this.uTexture, 1, width / 2, height / 2, uData);
        this.updateTexture(this.vTexture, 2, width / 2, height / 2, vData);
  
        // 清除画布并绘制
        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
    }
  }
  