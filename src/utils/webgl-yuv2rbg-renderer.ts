export class WebGLYUV2RGBRenderer {
    constructor(canvas) {
      this.canvas = canvas;
      this.gl = null;
      this.program = null;
      this.yTexture = null;
      this.uTexture = null;
      this.vTexture = null;
      this.initWebGL();
    }
  
    initWebGL() {
      this.gl = this.canvas.getContext("webgl");
      if (!this.gl) {
        console.error("WebGL 不受支持");
        return;
      }
  
      // 创建着色器和程序
      const vertexShader = this.createShader(this.gl.VERTEX_SHADER, vertexShaderSource);
      const fragmentShader = this.createShader(this.gl.FRAGMENT_SHADER, fragmentShaderSource);
      this.program = this.createProgram(vertexShader, fragmentShader);
      this.gl.useProgram(this.program);
  
      // 设置顶点和纹理坐标
      const vertices = new Float32Array([
        -1, -1, 0, 0,
        1, -1, 1, 0,
        -1, 1, 0, 1,
        1, 1, 1, 1,
      ]);
  
      const buffer = this.gl.createBuffer();
      this.gl.bindBuffer(this.gl.ARRAY_BUFFER, buffer);
      this.gl.bufferData(this.gl.ARRAY_BUFFER, vertices, this.gl.STATIC_DRAW);
  
      const positionLocation = this.gl.getAttribLocation(this.program, "position");
      const texcoordLocation = this.gl.getAttribLocation(this.program, "texcoord");
  
      this.gl.enableVertexAttribArray(positionLocation);
      this.gl.vertexAttribPointer(positionLocation, 2, this.gl.FLOAT, false, 16, 0);
  
      this.gl.enableVertexAttribArray(texcoordLocation);
      this.gl.vertexAttribPointer(texcoordLocation, 2, this.gl.FLOAT, false, 16, 8);
  
      // 创建 YUV 纹理
      this.yTexture = this.createTexture(0);
      this.uTexture = this.createTexture(1);
      this.vTexture = this.createTexture(2);
  
      // 设置纹理单元位置
      const yLocation = this.gl.getUniformLocation(this.program, "yTexture");
      const uLocation = this.gl.getUniformLocation(this.program, "uTexture");
      const vLocation = this.gl.getUniformLocation(this.program, "vTexture");
  
      this.gl.uniform1i(yLocation, 0);
      this.gl.uniform1i(uLocation, 1);
      this.gl.uniform1i(vLocation, 2);
    }
  
    createTexture(textureUnit) {
      const texture = this.gl.createTexture();
      this.gl.activeTexture(this.gl.TEXTURE0 + textureUnit);
      this.gl.bindTexture(this.gl.TEXTURE_2D, texture);
      this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_S, this.gl.CLAMP_TO_EDGE);
      this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_T, this.gl.CLAMP_TO_EDGE);
      this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MIN_FILTER, this.gl.LINEAR);
      this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MAG_FILTER, this.gl.LINEAR);
      return texture;
    }
  
    createShader(type, source) {
      const shader = this.gl.createShader(type);
      this.gl.shaderSource(shader, source);
      this.gl.compileShader(shader);
      if (!this.gl.getShaderParameter(shader, this.gl.COMPILE_STATUS)) {
        console.error("An error occurred compiling the shaders: " + this.gl.getShaderInfoLog(shader));
        this.gl.deleteShader(shader);
        return null;
      }
      return shader;
    }
  
    createProgram(vertexShader, fragmentShader) {
      const program = this.gl.createProgram();
      this.gl.attachShader(program, vertexShader);
      this.gl.attachShader(program, fragmentShader);
      this.gl.linkProgram(program);
      if (!this.gl.getProgramParameter(program, this.gl.LINK_STATUS)) {
        console.error("Unable to initialize the shader program: " + this.gl.getProgramInfoLog(program));
        return null;
      }
      return program;
    }
  
    renderFrame(yData, uData, vData, width, height) {
      if (!this.gl || !this.canvas) return;
  
      this.canvas.width = width;
      this.canvas.height = height;
      this.gl.viewport(0, 0, width, height);
  
      // 更新 Y 分量纹理
      this.gl.activeTexture(this.gl.TEXTURE0);
      this.gl.bindTexture(this.gl.TEXTURE_2D, this.yTexture);
      this.gl.texImage2D(
        this.gl.TEXTURE_2D,
        0,
        this.gl.LUMINANCE,
        width,
        height,
        0,
        this.gl.LUMINANCE,
        this.gl.UNSIGNED_BYTE,
        new Uint8Array(yData)
      );
  
      // 更新 U 分量纹理
      this.gl.activeTexture(this.gl.TEXTURE1);
      this.gl.bindTexture(this.gl.TEXTURE_2D, this.uTexture);
      this.gl.texImage2D(
        this.gl.TEXTURE_2D,
        0,
        this.gl.LUMINANCE,
        width / 2,
        height / 2,
        0,
        this.gl.LUMINANCE,
        this.gl.UNSIGNED_BYTE,
        new Uint8Array(uData)
      );
  
      // 更新 V 分量纹理
      this.gl.activeTexture(this.gl.TEXTURE2);
      this.gl.bindTexture(this.gl.TEXTURE_2D, this.vTexture);
      this.gl.texImage2D(
        this.gl.TEXTURE_2D,
        0,
        this.gl.LUMINANCE,
        width / 2,
        height / 2,
        0,
        this.gl.LUMINANCE,
        this.gl.UNSIGNED_BYTE,
        new Uint8Array(vData)
      );
  
      this.gl.drawArrays(this.gl.TRIANGLE_STRIP, 0, 4);
    }
}
  
const vertexShaderSource = `
  attribute vec2 position;
  attribute vec2 texcoord;
  varying vec2 v_texcoord;
  void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    // 修正纹理坐标的垂直翻转
    v_texcoord = vec2(texcoord.x, 1.0 - texcoord.y);
  }
`;

const fragmentShaderSource = `
  precision mediump float;
  varying vec2 v_texcoord;
  uniform sampler2D yTexture;
  uniform sampler2D uTexture;
  uniform sampler2D vTexture;

  void main() {
    // 获取 YUV 值
    float y = texture2D(yTexture, v_texcoord).r;
    float u = texture2D(uTexture, v_texcoord).r - 0.5;
    float v = texture2D(vTexture, v_texcoord).r - 0.5;

    // 使用 BT.601 标准的 YUV 到 RGB 转换
    // R = Y + 1.402 * V
    // G = Y - 0.344 * U - 0.714 * V
    // B = Y + 1.772 * U
    vec3 rgb;
    rgb.r = y + 1.402 * v;
    rgb.g = y - 0.344 * u - 0.714 * v;
    rgb.b = y + 1.772 * u;

    gl_FragColor = vec4(rgb, 1.0);
  }
`;