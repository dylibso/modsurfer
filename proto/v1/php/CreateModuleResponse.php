<?php
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# source: proto/v1/api.proto

use Google\Protobuf\Internal\GPBType;
use Google\Protobuf\Internal\RepeatedField;
use Google\Protobuf\Internal\GPBUtil;

/**
 * The message returned in response to a `CreateModuleRequest`.
 *
 * Generated from protobuf message <code>CreateModuleResponse</code>
 */
class CreateModuleResponse extends \Google\Protobuf\Internal\Message
{
    /**
     * Generated from protobuf field <code>int64 module_id = 1;</code>
     */
    protected $module_id = 0;
    /**
     * Generated from protobuf field <code>string hash = 2;</code>
     */
    protected $hash = '';
    /**
     * Generated from protobuf field <code>.Error error = 3;</code>
     */
    protected $error = null;

    /**
     * Constructor.
     *
     * @param array $data {
     *     Optional. Data for populating the Message object.
     *
     *     @type int|string $module_id
     *     @type string $hash
     *     @type \Error $error
     * }
     */
    public function __construct($data = NULL) {
        \GPBMetadata\Proto\V1\Api::initOnce();
        parent::__construct($data);
    }

    /**
     * Generated from protobuf field <code>int64 module_id = 1;</code>
     * @return int|string
     */
    public function getModuleId()
    {
        return $this->module_id;
    }

    /**
     * Generated from protobuf field <code>int64 module_id = 1;</code>
     * @param int|string $var
     * @return $this
     */
    public function setModuleId($var)
    {
        GPBUtil::checkInt64($var);
        $this->module_id = $var;

        return $this;
    }

    /**
     * Generated from protobuf field <code>string hash = 2;</code>
     * @return string
     */
    public function getHash()
    {
        return $this->hash;
    }

    /**
     * Generated from protobuf field <code>string hash = 2;</code>
     * @param string $var
     * @return $this
     */
    public function setHash($var)
    {
        GPBUtil::checkString($var, True);
        $this->hash = $var;

        return $this;
    }

    /**
     * Generated from protobuf field <code>.Error error = 3;</code>
     * @return \Error|null
     */
    public function getError()
    {
        return isset($this->error) ? $this->error : null;
    }

    public function hasError()
    {
        return isset($this->error);
    }

    public function clearError()
    {
        unset($this->error);
    }

    /**
     * Generated from protobuf field <code>.Error error = 3;</code>
     * @param \Error $var
     * @return $this
     */
    public function setError($var)
    {
        GPBUtil::checkMessage($var, \Error::class);
        $this->error = $var;

        return $this;
    }

}
